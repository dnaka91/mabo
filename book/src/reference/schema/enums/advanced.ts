enum SampleKind {
    Variant1,
    Variant2,
    // N variants...
}

interface SampleVariant1 {
    kind: SampleKind.Variant1;
    f1: number;
    f2: number;
}

interface SampleVariant2 {
    kind: SampleKind.Variant2;
    field1: number;
    field2: boolean;
}
