use std::any::type_name;

use pretty_assertions::assert_eq;

mod common;
use common::{quick_xml_deserialize_test, quick_xml_serialize_test};

use rstest::rstest;
use xmlity::{Deserialize, DeserializeOwned, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum EnumValue {
    Alpha,
    Beta,
    Gamma,
    ZuluZuluZulu,
    Iota_Iota_Iota,
    KAPPAKAPPAKAPPA,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "lowercase")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum EnumValuelower {
    Alpha,
    Beta,
    Gamma,
    ZuluZuluZulu,
    Iota_Iota_Iota,
    KAPPAKAPPAKAPPA,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "PascalCase")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum EnumValuePascalCase {
    Alpha,
    Beta,
    Gamma,
    ZuluZuluZulu,
    Iota_Iota_Iota,
    KAPPAKAPPAKAPPA,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "camelCase")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum EnumValueCamelCase {
    Alpha,
    Beta,
    Gamma,
    ZuluZuluZulu,
    Iota_Iota_Iota,
    KAPPAKAPPAKAPPA,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "snake_case")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum EnumValueSnakeCase {
    Alpha,
    Beta,
    Gamma,
    ZuluZuluZulu,
    Iota_Iota_Iota,
    KAPPAKAPPAKAPPA,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum EnumValueScreamingSnakeCase {
    Alpha,
    Beta,
    Gamma,
    ZuluZuluZulu,
    Iota_Iota_Iota,
    KAPPAKAPPAKAPPA,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "kebab-case")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum EnumValueKebabCase {
    Alpha,
    Beta,
    Gamma,
    ZuluZuluZulu,
    Iota_Iota_Iota,
    KAPPAKAPPAKAPPA,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "SCREAMING-KEBAB-CASE")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum EnumValueScreamingKebabCase {
    Alpha,
    Beta,
    Gamma,
    ZuluZuluZulu,
    Iota_Iota_Iota,
    KAPPAKAPPAKAPPA,
}
#[rstest]
#[case::default_alpha(EnumValue::Alpha, "Alpha")]
#[case::default_beta(EnumValue::Beta, "Beta")]
#[case::default_gamma(EnumValue::Gamma, "Gamma")]
#[case::default_zulu_zulu_zulu(EnumValue::ZuluZuluZulu, "ZuluZuluZulu")]
#[case::default_iota_iota_iota(EnumValue::Iota_Iota_Iota, "Iota_Iota_Iota")]
#[case::default_kappa_kappa_kappa(EnumValue::KAPPAKAPPAKAPPA, "KAPPAKAPPAKAPPA")]
#[case::lower_alpha(EnumValuelower::Alpha, "alpha")]
#[case::lower_beta(EnumValuelower::Beta, "beta")]
#[case::lower_gamma(EnumValuelower::Gamma, "gamma")]
#[case::lower_zulu_zulu_zulu(EnumValuelower::ZuluZuluZulu, "zuluzuluzulu")]
#[case::lower_iota_iota_iota(EnumValuelower::Iota_Iota_Iota, "iota_iota_iota")]
#[case::lower_kappa_kappa_kappa(EnumValuelower::KAPPAKAPPAKAPPA, "kappakappakappa")]
#[case::pascal_alpha(EnumValuePascalCase::Alpha, "Alpha")]
#[case::pascal_beta(EnumValuePascalCase::Beta, "Beta")]
#[case::pascal_gamma(EnumValuePascalCase::Gamma, "Gamma")]
#[case::pascal_zulu_zulu_zulu(EnumValuePascalCase::ZuluZuluZulu, "ZuluZuluZulu")]
#[case::pascal_iota_iota_iota(EnumValuePascalCase::Iota_Iota_Iota, "Iota_Iota_Iota")]
#[case::pascal_kappa_kappa_kappa(EnumValuePascalCase::KAPPAKAPPAKAPPA, "KAPPAKAPPAKAPPA")]
#[case::camel_alpha(EnumValueCamelCase::Alpha, "alpha")]
#[case::camel_beta(EnumValueCamelCase::Beta, "beta")]
#[case::camel_gamma(EnumValueCamelCase::Gamma, "gamma")]
#[case::camel_zulu_zulu_zulu(EnumValueCamelCase::ZuluZuluZulu, "zuluZuluZulu")]
#[case::camel_iota_iota_iota(EnumValueCamelCase::Iota_Iota_Iota, "iota_Iota_Iota")]
#[case::camel_kappa_kappa_kappa(EnumValueCamelCase::KAPPAKAPPAKAPPA, "kAPPAKAPPAKAPPA")]
#[case::snake_alpha(EnumValueSnakeCase::Alpha, "alpha")]
#[case::snake_beta(EnumValueSnakeCase::Beta, "beta")]
#[case::snake_gamma(EnumValueSnakeCase::Gamma, "gamma")]
#[case::snake_zulu_zulu_zulu(EnumValueSnakeCase::ZuluZuluZulu, "zulu_zulu_zulu")]
#[case::snake_iota_iota_iota(EnumValueSnakeCase::Iota_Iota_Iota, "iota__iota__iota")]
#[case::snake_kappa_kappa_kappa(
    EnumValueSnakeCase::KAPPAKAPPAKAPPA,
    "k_a_p_p_a_k_a_p_p_a_k_a_p_p_a"
)]
#[case::screaming_snake_alpha(EnumValueScreamingSnakeCase::Alpha, "ALPHA")]
#[case::screaming_snake_beta(EnumValueScreamingSnakeCase::Beta, "BETA")]
#[case::screaming_snake_gamma(EnumValueScreamingSnakeCase::Gamma, "GAMMA")]
#[case::screaming_snake_zulu_zulu_zulu(EnumValueScreamingSnakeCase::ZuluZuluZulu, "ZULU_ZULU_ZULU")]
#[case::screaming_snake_iota_iota_iota(
    EnumValueScreamingSnakeCase::Iota_Iota_Iota,
    "IOTA__IOTA__IOTA"
)]
#[case::screaming_snake_kappa_kappa_kappa(
    EnumValueScreamingSnakeCase::KAPPAKAPPAKAPPA,
    "K_A_P_P_A_K_A_P_P_A_K_A_P_P_A"
)]
#[case::kebab_case_alpha(EnumValueKebabCase::Alpha, "alpha")]
#[case::kebab_case_beta(EnumValueKebabCase::Beta, "beta")]
#[case::kebab_case_gamma(EnumValueKebabCase::Gamma, "gamma")]
#[case::kebab_case_zulu_zulu_zulu(EnumValueKebabCase::ZuluZuluZulu, "zulu-zulu-zulu")]
#[case::kebab_case_iota_iota_iota(EnumValueKebabCase::Iota_Iota_Iota, "iota--iota--iota")]
#[case::kebab_case_kappa_kappa_kappa(
    EnumValueKebabCase::KAPPAKAPPAKAPPA,
    "k-a-p-p-a-k-a-p-p-a-k-a-p-p-a"
)]
#[case::screaming_kebab_case_alpha(EnumValueScreamingKebabCase::Alpha, "ALPHA")]
#[case::screaming_kebab_case_beta(EnumValueScreamingKebabCase::Beta, "BETA")]
#[case::screaming_kebab_case_gamma(EnumValueScreamingKebabCase::Gamma, "GAMMA")]
#[case::screaming_kebab_case_zulu_zulu_zulu(
    EnumValueScreamingKebabCase::ZuluZuluZulu,
    "ZULU-ZULU-ZULU"
)]
#[case::screaming_kebab_case_iota_iota_iota(
    EnumValueScreamingKebabCase::Iota_Iota_Iota,
    "IOTA--IOTA--IOTA"
)]
#[case::screaming_kebab_case_kappa_kappa_kappa(
    EnumValueScreamingKebabCase::KAPPAKAPPAKAPPA,
    "K-A-P-P-A-K-A-P-P-A-K-A-P-P-A"
)]
fn serialize<T: Serialize + std::fmt::Debug>(#[case] value: T, #[case] expected: &str) {
    let actual = quick_xml_serialize_test(value).unwrap();

    assert_eq!(actual, expected);
}
#[rstest]
#[case::default_alpha(EnumValue::Alpha, "Alpha")]
#[case::default_beta(EnumValue::Beta, "Beta")]
#[case::default_gamma(EnumValue::Gamma, "Gamma")]
#[case::default_zulu_zulu_zulu(EnumValue::ZuluZuluZulu, "ZuluZuluZulu")]
#[case::default_iota_iota_iota(EnumValue::Iota_Iota_Iota, "Iota_Iota_Iota")]
#[case::default_kappa_kappa_kappa(EnumValue::KAPPAKAPPAKAPPA, "KAPPAKAPPAKAPPA")]
#[case::lower_alpha(EnumValuelower::Alpha, "alpha")]
#[case::lower_beta(EnumValuelower::Beta, "beta")]
#[case::lower_gamma(EnumValuelower::Gamma, "gamma")]
#[case::lower_zulu_zulu_zulu(EnumValuelower::ZuluZuluZulu, "zuluzuluzulu")]
#[case::lower_iota_iota_iota(EnumValuelower::Iota_Iota_Iota, "iota_iota_iota")]
#[case::lower_kappa_kappa_kappa(EnumValuelower::KAPPAKAPPAKAPPA, "kappakappakappa")]
#[case::pascal_alpha(EnumValuePascalCase::Alpha, "Alpha")]
#[case::pascal_beta(EnumValuePascalCase::Beta, "Beta")]
#[case::pascal_gamma(EnumValuePascalCase::Gamma, "Gamma")]
#[case::pascal_zulu_zulu_zulu(EnumValuePascalCase::ZuluZuluZulu, "ZuluZuluZulu")]
#[case::pascal_iota_iota_iota(EnumValuePascalCase::Iota_Iota_Iota, "Iota_Iota_Iota")]
#[case::pascal_kappa_kappa_kappa(EnumValuePascalCase::KAPPAKAPPAKAPPA, "KAPPAKAPPAKAPPA")]
#[case::camel_alpha(EnumValueCamelCase::Alpha, "alpha")]
#[case::camel_beta(EnumValueCamelCase::Beta, "beta")]
#[case::camel_gamma(EnumValueCamelCase::Gamma, "gamma")]
#[case::camel_zulu_zulu_zulu(EnumValueCamelCase::ZuluZuluZulu, "zuluZuluZulu")]
#[case::camel_iota_iota_iota(EnumValueCamelCase::Iota_Iota_Iota, "iota_Iota_Iota")]
#[case::camel_kappa_kappa_kappa(EnumValueCamelCase::KAPPAKAPPAKAPPA, "kAPPAKAPPAKAPPA")]
#[case::snake_alpha(EnumValueSnakeCase::Alpha, "alpha")]
#[case::snake_beta(EnumValueSnakeCase::Beta, "beta")]
#[case::snake_gamma(EnumValueSnakeCase::Gamma, "gamma")]
#[case::snake_zulu_zulu_zulu(EnumValueSnakeCase::ZuluZuluZulu, "zulu_zulu_zulu")]
#[case::snake_iota_iota_iota(EnumValueSnakeCase::Iota_Iota_Iota, "iota__iota__iota")]
#[case::snake_kappa_kappa_kappa(
    EnumValueSnakeCase::KAPPAKAPPAKAPPA,
    "k_a_p_p_a_k_a_p_p_a_k_a_p_p_a"
)]
#[case::screaming_snake_alpha(EnumValueScreamingSnakeCase::Alpha, "ALPHA")]
#[case::screaming_snake_beta(EnumValueScreamingSnakeCase::Beta, "BETA")]
#[case::screaming_snake_gamma(EnumValueScreamingSnakeCase::Gamma, "GAMMA")]
#[case::screaming_snake_zulu_zulu_zulu(EnumValueScreamingSnakeCase::ZuluZuluZulu, "ZULU_ZULU_ZULU")]
#[case::screaming_snake_iota_iota_iota(
    EnumValueScreamingSnakeCase::Iota_Iota_Iota,
    "IOTA__IOTA__IOTA"
)]
#[case::screaming_snake_kappa_kappa_kappa(
    EnumValueScreamingSnakeCase::KAPPAKAPPAKAPPA,
    "K_A_P_P_A_K_A_P_P_A_K_A_P_P_A"
)]
#[case::kebab_case_alpha(EnumValueKebabCase::Alpha, "alpha")]
#[case::kebab_case_beta(EnumValueKebabCase::Beta, "beta")]
#[case::kebab_case_gamma(EnumValueKebabCase::Gamma, "gamma")]
#[case::kebab_case_zulu_zulu_zulu(EnumValueKebabCase::ZuluZuluZulu, "zulu-zulu-zulu")]
#[case::kebab_case_iota_iota_iota(EnumValueKebabCase::Iota_Iota_Iota, "iota--iota--iota")]
#[case::kebab_case_kappa_kappa_kappa(
    EnumValueKebabCase::KAPPAKAPPAKAPPA,
    "k-a-p-p-a-k-a-p-p-a-k-a-p-p-a"
)]
#[case::screaming_kebab_case_alpha(EnumValueScreamingKebabCase::Alpha, "ALPHA")]
#[case::screaming_kebab_case_beta(EnumValueScreamingKebabCase::Beta, "BETA")]
#[case::screaming_kebab_case_gamma(EnumValueScreamingKebabCase::Gamma, "GAMMA")]
#[case::screaming_kebab_case_zulu_zulu_zulu(
    EnumValueScreamingKebabCase::ZuluZuluZulu,
    "ZULU-ZULU-ZULU"
)]
#[case::screaming_kebab_case_iota_iota_iota(
    EnumValueScreamingKebabCase::Iota_Iota_Iota,
    "IOTA--IOTA--IOTA"
)]
#[case::screaming_kebab_case_kappa_kappa_kappa(
    EnumValueScreamingKebabCase::KAPPAKAPPAKAPPA,
    "K-A-P-P-A-K-A-P-P-A-K-A-P-P-A"
)]
fn deserialize<T: xmlity::DeserializeOwned + std::fmt::Debug + PartialEq>(
    #[case] expected: T,
    #[case] text: &str,
) {
    let actual: T = quick_xml_deserialize_test(text).unwrap();

    assert_eq!(actual, expected);
}

#[rstest]
#[case::default_0("Lmao", EnumValue::Alpha)]
#[case::default_1("lmao", EnumValue::Alpha)]
#[case::default_2("BETA", EnumValue::Alpha)]
#[case::default_3("gamma", EnumValue::Alpha)]
#[case::default_4("zuluZuluZulu", EnumValue::Alpha)]
#[case::default_5("IOTA_IOTA_IOTA", EnumValue::Alpha)]
#[case::default_6("kappakappakappa", EnumValue::Alpha)]
#[case::lower_0("ALPHA", EnumValuelower::Alpha)]
#[case::lower_1("Beta", EnumValuelower::Alpha)]
#[case::lower_2("GAMMA", EnumValuelower::Alpha)]
#[case::lower_3("ZuluZuluZulu", EnumValuelower::Alpha)]
#[case::lower_4("iotaiotaiota", EnumValuelower::Alpha)]
#[case::lower_5("KAPPAKAPPAKAPPA", EnumValuelower::Alpha)]
#[case::pascal_0("alpha", EnumValuePascalCase::Alpha)]
#[case::pascal_1("BETA", EnumValuePascalCase::Alpha)]
#[case::pascal_2("gamma", EnumValuePascalCase::Alpha)]
#[case::pascal_3("zuluzuluzulu", EnumValuePascalCase::Alpha)]
#[case::pascal_4("IotaIotaIota", EnumValuePascalCase::Alpha)]
#[case::pascal_5("KappaKappaKappa", EnumValuePascalCase::Alpha)]
#[case::camel_0("ALPHA", EnumValueCamelCase::Alpha)]
#[case::camel_1("geta", EnumValueCamelCase::Alpha)]
#[case::camel_2("GAMMA", EnumValueCamelCase::Alpha)]
#[case::camel_3("ZuluZuluZulu", EnumValueCamelCase::Alpha)]
#[case::camel_4("iotaIotaIota", EnumValueCamelCase::Alpha)]
#[case::camel_5("kappaKappaKappa", EnumValueCamelCase::Alpha)]
#[case::snake_0("Alpha", EnumValueSnakeCase::Alpha)]
#[case::snake_1("BETA", EnumValueSnakeCase::Alpha)]
#[case::snake_2("GAMMA", EnumValueSnakeCase::Alpha)]
#[case::snake_3("zuluZuluZulu", EnumValueSnakeCase::Alpha)]
#[case::snake_4("IOTA_IOTA_IOTA", EnumValueSnakeCase::Alpha)]
#[case::snake_5("kappa_kappa_kappa", EnumValueSnakeCase::Alpha)]
#[case::screaming_snake_0("alpha", EnumValueScreamingSnakeCase::Alpha)]
#[case::screaming_snake_1("beta", EnumValueScreamingSnakeCase::Alpha)]
#[case::screaming_snake_2("Gamma", EnumValueScreamingSnakeCase::Alpha)]
#[case::screaming_snake_3("ZULU-ZULU-ZULU", EnumValueScreamingSnakeCase::Alpha)]
#[case::screaming_snake_4("iota_iota_iota", EnumValueScreamingSnakeCase::Alpha)]
#[case::screaming_snake_5("KAPPA_KAPPA_KAPPA", EnumValueScreamingSnakeCase::Alpha)]
#[case::kebab_0("Alpha", EnumValueKebabCase::Alpha)]
#[case::kebab_1("BETA", EnumValueKebabCase::Alpha)]
#[case::kebab_2("Gamma", EnumValueKebabCase::Alpha)]
#[case::kebab_3("zulu_zulu_zulu", EnumValueKebabCase::Alpha)]
#[case::kebab_4("IOTA-IOTA-IOTA", EnumValueKebabCase::Alpha)]
#[case::kebab_5("kappa-kappa-kappa", EnumValueKebabCase::Alpha)]
#[case::screaming_kebab_0("alpha", EnumValueScreamingKebabCase::Alpha)]
#[case::screaming_kebab_1("beta", EnumValueScreamingKebabCase::Alpha)]
#[case::screaming_kebab_2("Gamma", EnumValueScreamingKebabCase::Alpha)]
#[case::screaming_kebab_3("ZULU_ZULU_ZULU", EnumValueScreamingKebabCase::Alpha)]
#[case::screaming_kebab_4("IOTA-IOTA-IOTA", EnumValueScreamingKebabCase::Alpha)]
#[case::screaming_kebab_5("KAPPA-KAPPA-KAPPA", EnumValueScreamingKebabCase::Alpha)]
fn wrong_deserialize<T: DeserializeOwned + std::fmt::Debug + PartialEq>(
    #[case] invalid: &str,
    #[case] _type_marker: T,
) {
    let actual: Result<T, _> = quick_xml_deserialize_test(invalid);
    assert!(actual.is_err());
    if let xmlity_quick_xml::Error::NoPossibleVariant { ident } = actual.unwrap_err() {
        assert_eq!(ident, type_name::<T>().split("::").last().unwrap());
        println!("{:?}", ident);
    } else {
        panic!("Unexpected error type");
    }
}
