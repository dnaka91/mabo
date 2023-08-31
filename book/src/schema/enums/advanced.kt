fun main() {}

sealed class Sample {
    class Variant1(
        val f1: UInt,
        val f2: UShort,
    ) : Sample()

    class Variant2(
        val field1: Long,
        val field2: Boolean,
    ) : Sample()

    // N variants...
}
