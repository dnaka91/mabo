package main

func main() {}

type Sample struct {
	F1 uint32
	F2 uint16
}

func NewSample(f1 uint32, f2 uint16) Sample {
	return Sample{
		F1: f1,
		F2: f2,
	}
}
