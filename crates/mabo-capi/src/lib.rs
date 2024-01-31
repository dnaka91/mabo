#![allow(
    missing_docs,
    unsafe_code,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::{
    ffi::{CStr, CString, OsStr, c_char},
    os::unix::ffi::OsStrExt,
    path::Path,
    ptr::NonNull,
};

#[cfg_attr(test, derive(Debug, PartialEq))]
#[repr(C)]
pub enum ErrorCode {
    Ok = 0,
    Unknown = 1,
}

pub struct ProjectList(Vec<Project>);

#[unsafe(no_mangle)]
pub extern "C" fn mabo_project_discover(path: &c_char, out: &mut *mut ProjectList) -> ErrorCode {
    let path = unsafe { CStr::from_ptr(path) };
    let path = OsStr::from_bytes(path.to_bytes());

    let Ok(project) = mabo_project::discover(path) else {
        return ErrorCode::Unknown;
    };

    let ptr = Box::leak(Box::new(ProjectList(
        project.into_iter().map(Project).collect(),
    )));

    *out = ptr;

    ErrorCode::Ok
}

#[unsafe(no_mangle)]
pub extern "C" fn mabo_project_list_free(ptr: NonNull<ProjectList>) {
    drop(unsafe { Box::from_raw(ptr.as_ptr()) });
}

#[unsafe(no_mangle)]
pub extern "C" fn mabo_project_list_len(list: &ProjectList) -> usize {
    list.0.len()
}

#[unsafe(no_mangle)]
pub extern "C" fn mabo_project_list_get(list: &ProjectList, index: usize) -> *const Project {
    &list.0[index]
}

pub struct Project(mabo_project::Project);

#[unsafe(no_mangle)]
pub extern "C" fn mabo_project_load(path: &c_char, out: &mut *mut Project) -> ErrorCode {
    let path = unsafe { CStr::from_ptr(path) };
    let path = OsStr::from_bytes(path.to_bytes());

    let Ok(project) = mabo_project::load(path) else {
        return ErrorCode::Unknown;
    };

    let ptr = Box::leak(Box::new(Project(project)));

    *out = ptr;

    ErrorCode::Ok
}

#[unsafe(no_mangle)]
pub extern "C" fn mabo_project_free(ptr: NonNull<Project>) {
    drop(unsafe { Box::from_raw(ptr.as_ptr()) });
}

#[derive(Debug)]
pub struct Schema<'a>(mabo_parser::Schema<'a>);

#[unsafe(no_mangle)]
pub extern "C" fn mabo_schema_parse<'a>(
    input: &'a c_char,
    path: Option<&'a c_char>,
    out: &mut *mut Schema<'a>,
) -> ErrorCode {
    let input = unsafe { CStr::from_ptr(input) };
    let Ok(input) = input.to_str() else {
        return ErrorCode::Unknown;
    };

    let path = match path {
        Some(path) => {
            let path = unsafe { CStr::from_ptr(path) };
            let Ok(path) = path.to_str() else {
                return ErrorCode::Unknown;
            };
            Some(Path::new(path))
        }
        None => None,
    };

    let Ok(schema) = mabo_parser::Schema::parse(input, path) else {
        return ErrorCode::Unknown;
    };

    let ptr = Box::leak(Box::new(Schema(schema)));

    *out = ptr;

    ErrorCode::Ok
}

#[unsafe(no_mangle)]
pub extern "C" fn mabo_schema_free(ptr: NonNull<Schema<'_>>) {
    drop(unsafe { Box::from_raw(ptr.as_ptr()) });
}

#[unsafe(no_mangle)]
pub extern "C" fn mabo_schema_validate(schema: &Schema<'_>) -> bool {
    mabo_compiler::validate_schema(&schema.0).is_ok()
}

pub struct SimplifiedSchema<'a>(mabo_compiler::simplify::Schema<'a>);

#[unsafe(no_mangle)]
pub extern "C" fn mabo_schema_simplify<'a>(schema: &'a Schema<'_>) -> *mut SimplifiedSchema<'a> {
    let simplified = mabo_compiler::simplify_schema(&schema.0);

    Box::leak(Box::new(SimplifiedSchema(simplified)))
}

#[unsafe(no_mangle)]
pub extern "C" fn mabo_simplified_schema_free(ptr: NonNull<SimplifiedSchema<'_>>) {
    drop(unsafe { Box::from_raw(ptr.as_ptr()) });
}

/// Turn the simplified schema into a JSON string.
///
/// Note: Make sure to release the returned string with `mabo_string_free()` and **not** with the
/// regular `free()` function.
///
/// @param[in] schema Valid non-null pointer to a simplified schema.
/// @return JSON stringified representation of the schema.
#[unsafe(no_mangle)]
pub extern "C" fn mabo_simplified_schema_to_json(schema: &SimplifiedSchema<'_>) -> *mut c_char {
    let json = serde_json::to_string(&schema.0).unwrap();
    let json = CString::new(json).unwrap();

    json.into_raw()
}

/// Free any string created with the mabo library (currently only
/// `mabo_simplified_schema_to_json()`).
///
/// @param[in] ptr Valid non-null pointer to a string.
#[unsafe(no_mangle)]
pub extern "C" fn mabo_string_free(ptr: NonNull<c_char>) {
    drop(unsafe { CString::from_raw(ptr.as_ptr()) });
}

#[cfg(test)]
mod tests {
    use std::{ffi::CStr, ptr::NonNull};

    use crate::{ErrorCode, Schema, SimplifiedSchema};

    #[test]
    fn parse() {
        let input = b"struct Sample { value: u32 @1 }\0";
        let path = None;
        let mut schema = std::ptr::null_mut();

        let code = super::mabo_schema_parse(unsafe { &*input.as_ptr().cast() }, path, &mut schema);
        assert_eq!(ErrorCode::Ok, code);

        let schema = unsafe { &*schema };
        assert_eq!(
            "struct Sample {\n    value: u32 @1,\n}\n\n",
            schema.0.to_string()
        );

        super::mabo_schema_free(schema.into());
    }

    #[test]
    fn validate() {
        let schema =
            Schema(mabo_parser::Schema::parse("struct Sample { value: u32 @1 }", None).unwrap());

        assert!(super::mabo_schema_validate(&schema));
    }

    #[test]
    fn simplify() {
        let schema =
            Schema(mabo_parser::Schema::parse("struct Sample { value: u32 @1 }", None).unwrap());

        let simplified = super::mabo_schema_simplify(&schema);
        super::mabo_simplified_schema_free(NonNull::new(simplified).unwrap());
    }

    #[test]
    fn to_json() {
        let schema = mabo_parser::Schema::parse("struct Sample { value: u32 @1 }", None).unwrap();
        let schema = SimplifiedSchema(mabo_compiler::simplify_schema(&schema));

        let json = super::mabo_simplified_schema_to_json(&schema);
        let value = serde_json::from_str::<serde_json::Value>(
            unsafe { CStr::from_ptr(json) }.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(
            serde_json::json!({
                "comment": [],
                "definitions": [{
                    "Struct": {
                        "comment": [],
                        "name": "Sample",
                        "generics": [],
                        "fields": {
                            "fields": [{
                                "comment": [],
                                "name": "value",
                                "ty": "U32",
                                "id": 1
                            }],
                            "kind": "Named"
                        }
                    }
                }]
            }),
            value,
        );

        super::mabo_string_free(NonNull::new(json).unwrap());
    }
}
