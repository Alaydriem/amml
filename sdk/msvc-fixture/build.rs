fn main() {
    println!("cargo:rerun-if-changed=cpp/fixture.cpp");
    cc::Build::new()
        .cpp(true)
        .file("cpp/fixture.cpp")
        .flag("/O2")
        .flag("/MD")
        .flag("/EHsc")
        .flag("/std:c++20")
        .define("_ITERATOR_DEBUG_LEVEL", "0")
        .opt_level(2)
        .debug(false)
        .cargo_warnings(false)
        .compile("amml_msvc_fixture");
}
