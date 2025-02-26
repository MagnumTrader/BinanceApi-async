#![allow(unused, unreachable_code)]
/// När jag gör matching för att trigga olika beteenden kan det vara fördelaktigt att använda enum,
/// och ha en function som implementerar display istället
/// detta är för om det är på flera ställen, om den dock används som en wrapper typ, och vi bara
/// vill kontrollera vilka olika värden som kan skickas in, använd const!




#[derive(Debug)]
pub struct Timeframe(&'static str);

impl std::fmt::Display for Timeframe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
struct MyWrapper(i32);

fn stuff(a: MyWrapper, b: i32) {

    let c = *a+b;
}
impl std::ops::Deref for MyWrapper {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Timeframe {
    pub const ONEMINUTE: Timeframe = Timeframe("1m");
    pub const ONEHOUR: Timeframe = Timeframe("1h");
    const ONEDAY: Timeframe = Timeframe("1d");
}

