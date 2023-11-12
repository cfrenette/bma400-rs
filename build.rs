#[rustversion::since(1.76.0)]
fn main() {}

#[rustversion::all(before(1.76.0), nightly)]
fn main() {
    #[cfg(feature = "async")]
    {
        println!("cargo:rustc-cfg=enable_async_in_trait");
    }
}

#[rustversion::all(before(1.76.0), not(nightly))]
fn main() {
    #[cfg(feature = "async")]
    {
        println!("cargo:warning=async support requires rustc 1.76.0 or newer or a nightly compiler supporting the `async_fn_in_trait` feature");
    }
}
