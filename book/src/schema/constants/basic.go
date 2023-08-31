package main

import "math/big"

func main() {}

const (
	ValueBool bool = true

	ValueU8  uint8  = 1
	ValueU16 uint16 = 2
	ValueU32 uint32 = 3
	ValueU64 uint64 = 4

	ValueI8  int8  = -1
	ValueI16 int16 = -2
	ValueI32 int32 = -3
	ValueI64 int64 = -4

	ValueF32 float32 = 1.0
	ValueF64 float64 = 2.0

	ValueStr string = "abc"
)

var (
	ValueU128  *big.Int = big.NewInt(5)
	ValueI128  *big.Int = big.NewInt(-5)
	ValueBytes []byte   = []byte{1, 2, 3}
)
