package main

func main() {}

type SampleVariant interface {
	sealed()
}

type Sample SampleVariant

type Sample_Variant1 struct {
	F1 uint32
	F2 uint16
}

func (v Sample_Variant1) sealed() {}

type Sample_Variant2 struct {
	Field1 int64
	Field2 bool
}

func (v Sample_Variant2) sealed() {}

// N variants...
