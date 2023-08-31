package main

func main() {}

type Tuple2[T1 any, T2 any] struct {
	F1 T1
	F2 T2
}

/*
type Tuple3[T1 any, T2 any, T3 any] struct {
	F1 T1
	F2 T2
	F2 T3
}

...and so on...
*/

type Sample struct {
	Size Tuple2[uint32, uint32]
}
