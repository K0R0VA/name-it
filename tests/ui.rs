#[test]
fn ui() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}


#[name_it::async_trait]
pub trait SomeAsyncTrait {
    async fn bar(&self) -> i32;
}

struct Foo;

#[name_it::async_trait]
impl SomeAsyncTrait for Foo {
    async fn bar(&self) -> i32 {
        0
    }
}

#[tokio::test]
async fn test() {
    let foo = Foo;
    assert_eq!(0, foo.bar().await)
}

