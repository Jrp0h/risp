pub mod vm;

// macro_rules! variants {
//     () => {
//         [Variant::None, Variant::None, Variant::None]
//     };
//     ($var:ident) => {
//         [Variant::$var, Variant::None, Variant::None]
//     };
//     ($var1:ident, $var2:ident) => {
//         [Variant::$var1, Variant::$var2, Variant::None]
//     };
//     ($var1:ident, $var2:ident, $var3:ident) => {
//         [Variant::$var1, Variant::$var2, Variant::$var3]
//     };
// }

// macro_rules! op {
//     ($op:ident) => {
//         OpCode::new(Operation::$op, variants!()).as_usize()
//     };
//     ($op:ident, $($vars:ident),+) => {
//         OpCode::new(Operation::$op, variants!($($vars),*)).as_usize()
//     };
// }
