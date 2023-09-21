#!ic-repl

let b = file("arr.txt");  // a.txt contains 768 bytes
function f(x) { let _ = 0.1 };
let b = b.map(f);

let vdb = service "bkyz2-fmaaa-aaaaa-qaaaq-cai";
call vdb.get_similar(0, b, 1);