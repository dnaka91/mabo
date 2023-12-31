fun main() {}

// #region snippet
data class Tuple2<T1, T2>(
    val f1: T1,
    val f2: T2,
)

/*
data class Tuple3<T1, T2, T3>(
    val f1: T1,
    val f2: T2,
    val f3: T3
)

...and so on...
*/

data class Sample(
    val size: Tuple2<UInt, UInt>,
)
// #endregion snippet
