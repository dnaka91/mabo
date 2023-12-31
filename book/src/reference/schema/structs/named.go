package main

func main() {}

// #region snippet
type Sample struct {
	Field1 uint32
	Field2 uint16
	// N fields...
}

func NewSample(field1 uint32, field2 uint16) Sample {
	return Sample{
		Field1: field1,
		Field2: field2,
	}
}

// #endregion snippet
