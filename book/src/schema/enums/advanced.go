package main

func main() {}

type SampleVariant interface {
	sealed()
}

type Sample SampleVariant

type SampleVariant1 struct {
	F1 uint32
	F2 uint16
}

func (v SampleVariant1) sealed() {}

type SampleVariant2 struct {
	Field1 int64
	Field2 bool
}

func (v SampleVariant2) sealed() {}

// N variants...
