# Zetro

Generate typed and extremely efficient APIs from a schema file. Effectively turning an API response like this:

```json
// 735 bytes: key-value pair
[[["YiGepyIChwIjKAW1XFFbSD-DH-4",{"rooms":[{"id":0,"messages":[{"author":{"username":"hal42"},"date":1639903136,"id":192,"text":"cats are fun!"},{"author":{"username":"droopydifferential"},"date":1639898582,"id":23489,"text":"perhaps, but have you tried solving differential equations?"}],"name":"Furry cats","status":0},{"id":1,"messages":[{"author":{"username":"mitoch0ndria"},"date":1639904622,"id":3489,"text":"...so I told them to watch 3b1b..."},{"author":{"username":"droopydifferential"},"date":1639907197,"id":1290,"text":"that is indeed quite entertaining to hear."},{"author":{"username":"mitoch0ndria"},"date":1639907197,"id":2390,"text":"[mitoch0ndria left the room]"}],"name":"Differential calculus","status":0}]}]],null]
```

into this:

```json
// 468 bytes: untagged array representation
[[["YiGepyIChwIjKAW1XFFbSD-DH-4",[[[0,[[["hal42"],1639903351,192,"cats are fun!"],[["droopydifferential"],1639898797,23489,"perhaps, but have you tried solving differential equations?"]],"Furry cats",0],[1,[[["mitoch0ndria"],1639904837,3489,"...so I told them to watch 3b1b..."],[["droopydifferential"],1639907412,1290,"that is indeed quite entertaining to hear."],[["mitoch0ndria"],1639907412,2390,"[mitoch0ndria left the room]"]],"Differential calculus",0]]]]],null]
```

## ⚠️ Warning ⚠️

Zetro currently generates only server code for Warp (Rust) and client code for TypeScript.
PRs for other languages are welcome!

## Getting Started

You need Cargo v1.56.0 or later to build Zetro. You can also use the binary releases

```bash
export REPO_FOLDER="zetro"

# Acquire the code
$ git clone https://github.com/muscache/zetro --depth 1 $REPO_FOLDER

# Build the tool
$ cd $REPO_FOLDER/zetro
$ cargo build

# Add zetro to PATH (recommended)
$ mv target/debug/zetro ~/.local/bin # Or move to /usr/local/bin

# Verify Zetro is installed
$ zetro # Should get 'Missing option --schema'

# Ready to run the example!
```

## Running the example

Ensure you have installed Zetro correctly before proceeding.

```bash
$ cd $REPO_FOLDER/example

# Generate Rust and TypeScript code
./generate_code.sh

# Transpile + minify the client code
cd client
npm install
npm run build

# Start the API server
cd ../server
cargo run

# Visit http://localhost:8090 in your browser
```

## Why?

This tool solves multiple problems:

- **Typed APIs**: Ensures client and server payloads are always the type and shape you define.
- **Efficiency**: Tiny payloads. Only transfer raw data, not keys.
- **Obfuscation**: Not a primary goal, but Zetro does make it harder for third parties to reverse engineer or use your APIs.

Of course, GZipping already minifies API responses when using "normal" JSON, but Zetro takes it one step further by also
reducing allocations (ie., allocation of keys in the object) when unpacking the response.
