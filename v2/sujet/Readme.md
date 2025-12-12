# Martin Tom, Roy Tom, André Léon

cargo install --path ../rvm_01
RVM_LOG=trace cargo run exec ../examples2/ok/test_100.tasm -vvv 
RVM_LOG=trace cargo run exec ../examples/ko/test_1.tasm -vvv