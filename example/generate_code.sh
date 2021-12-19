../zetro/target/debug/zetro --schema=./schema.json --out-file=./server/src/generated/code_generated.rs --add-plugin='warp(fnv:true)' --untagged=true
../zetro/target/debug/zetro --schema=./schema.json --out-file=./client/src/generated/code_generated.ts --add-plugin='class-client' --untagged=true
