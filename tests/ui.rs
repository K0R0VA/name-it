#[test]
fn ui() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}


#[name_it::async_trait]
pub trait SomeAsyncTrait {
    async fn bar(&self) -> i32;
    async fn inc(&self, i: i32 ) -> i32;
}

struct Foo;

#[name_it::async_trait]
impl SomeAsyncTrait for Foo {
    async fn bar(&self) -> i32 {
        0
    }
    async fn inc(&self, i: i32 ) -> i32 {
        i + 1
    }
}

#[tokio::test]
async fn test() {
    let foo = Foo;
    assert_eq!(0, foo.bar().await);
    assert_eq!(1, foo.inc(0).await);
}

