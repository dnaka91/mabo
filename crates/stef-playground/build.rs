fn main() {
    stef_build::compile(&["src/sample.stef"], &["src/"]).unwrap();
    stef_build::compile(&["schemas/*.stef"], &["schemas/"]).unwrap();
}
