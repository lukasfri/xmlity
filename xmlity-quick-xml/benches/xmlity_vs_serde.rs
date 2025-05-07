use criterion::criterion_main;

mod xmlity_bench {
    use criterion::{black_box, criterion_group, Criterion};
    use xmlity::{Deserialize, DeserializeOwned, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    #[xelement(name = "b")]
    pub struct B(pub String);

    #[derive(Clone, Serialize, Deserialize)]
    #[xelement(name = "c")]
    pub struct C {
        pub b: B,
    }

    #[derive(Serialize, Deserialize)]
    #[xelement(name = "d")]
    pub struct D {
        pub c: Vec<C>,
    }

    pub fn deserialize<T: DeserializeOwned>(xml: &str) -> T {
        xmlity_quick_xml::de::from_str(xml).unwrap()
    }

    pub fn serialize<T: Serialize>(value: &T) -> String {
        xmlity_quick_xml::ser::to_string(value).unwrap()
    }

    fn criterion_benchmark(c: &mut Criterion) {
        c.bench_function("xmlity basic deserialize", |b| {
            b.iter(|| deserialize::<C>(black_box(r#"<c><b>A</b></c>"#)))
        })
        .bench_function("xmlity basic serialize", |b| {
            b.iter(|| {
                serialize(&C {
                    b: B("A".to_string()),
                })
            })
        })
        .bench_function("xmlity list deserialize", |b| {
            let constructed_xml = format!(
                r#"<d>{}</d>"#,
                (0..50)
                    .map(|_| r#"<c><b>A</b></c>"#)
                    .collect::<Vec<_>>()
                    .join("")
            );
            b.iter(|| deserialize::<D>(black_box(constructed_xml.as_str())))
        })
        .bench_function("xmlity list serialize", |b| {
            b.iter(|| {
                serialize(&D {
                    c: vec![
                        C {
                            b: B("A".to_string()),
                        };
                        50
                    ],
                })
            })
        });
    }
    criterion_group!(benches, criterion_benchmark);
}

mod serde_bench {
    use criterion::{black_box, criterion_group, Criterion};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    pub struct C {
        pub b: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct D {
        pub c: Vec<C>,
    }

    pub fn deserialize<C: DeserializeOwned>(xml: &str) -> C {
        quick_xml::de::from_str(xml).unwrap()
    }

    pub fn serialize<T: Serialize>(value: &T) -> String {
        quick_xml::se::to_string(value).unwrap()
    }

    fn criterion_benchmark(c: &mut Criterion) {
        c.bench_function("serde basic deserialize", |b| {
            b.iter(|| deserialize::<C>(black_box(r#"<c><b>A</b></c>"#)))
        })
        .bench_function("serde basic serialize", |b| {
            b.iter(|| serialize(&C { b: "A".to_string() }))
        })
        .bench_function("serde list deserialize", |b| {
            let constructed_xml = format!(
                r#"<d>{}</d>"#,
                (0..50)
                    .map(|_| r#"<c><b>A</b></c>"#)
                    .collect::<Vec<_>>()
                    .join("")
            );
            b.iter(|| deserialize::<D>(black_box(constructed_xml.as_str())))
        })
        .bench_function("serde list serialize", |b| {
            b.iter(|| {
                serialize(&D {
                    c: vec![C { b: "A".to_string() }; 50],
                })
            })
        });
    }
    criterion_group!(benches, criterion_benchmark);
}

mod yaserde_bench {
    use criterion::{black_box, criterion_group, Criterion};
    use yaserde::{YaDeserialize, YaSerialize};

    #[derive(Clone, YaSerialize, YaDeserialize)]
    #[yaserde(rename = "c")]
    pub struct C {
        #[yaserde(rename = "b")]
        pub b: String,
    }

    #[derive(YaSerialize, YaDeserialize)]
    #[yaserde(rename = "d")]
    pub struct D {
        pub c: Vec<C>,
    }

    pub fn deserialize<C: YaDeserialize>(xml: &str) -> C {
        yaserde::de::from_str(xml).unwrap()
    }

    pub fn serialize<T: YaSerialize>(value: &T) -> String {
        yaserde::ser::to_string(value).unwrap()
    }

    fn criterion_benchmark(c: &mut Criterion) {
        c.bench_function("yaserde basic deserialize", |b| {
            b.iter(|| deserialize::<C>(black_box(r#"<c><b>A</b></c>"#)))
        })
        .bench_function("yaserde basic serialize", |b| {
            b.iter(|| serialize(&C { b: "A".to_string() }))
        })
        .bench_function("yaserde list deserialize", |b| {
            let constructed_xml = format!(
                r#"<d>{}</d>"#,
                (0..50)
                    .map(|_| r#"<c><b>A</b></c>"#)
                    .collect::<Vec<_>>()
                    .join("")
            );
            b.iter(|| deserialize::<D>(black_box(constructed_xml.as_str())))
        })
        .bench_function("yaserde list serialize", |b| {
            b.iter(|| {
                serialize(&D {
                    c: vec![C { b: "A".to_string() }; 50],
                })
            })
        });
    }
    criterion_group!(benches, criterion_benchmark);
}

criterion_main!(
    xmlity_bench::benches,
    serde_bench::benches,
    yaserde_bench::benches
);
