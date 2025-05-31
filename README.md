# Ser

Serializability checker.

## File Structure

- `src/main.rs`: Entry point of the application that handles command line arguments and processes files.
- `src/parser.rs`: Parses the serializable expressions (`.ser` files).
- `src/ns.rs`: Implements the Network System (NS) data structure.
- `src/expr_to_ns.rs`: Converts expressions to Network Systems.
- `src/ns_to_petri.rs`: Converts Network Systems to Petri nets.
- `src/kleene.rs`: Implements Kleene algebra operations and Kleene's algorithm.
- `src/semilinear.rs`: Contains semilinear set operations and implements the Kleene trait.
- `src/petri.rs`: Implements Petri net data structures.
- `src/graphviz.rs`: Handles visualization of data structures.
- `src/isl.rs`: Wrapper around the ISL library.

- `examples/json/*`: NS (Network System) examples with directly specified automaton of requests, transitions, responses.
- `examples/ser/*`: Examples specified in the Ser programming language.

- `out/*`: Output visualizations.

- `.vscode/*`: VSCode configuration files for syntax highlighting and editor settings.

## Dependencies

Depends on [isl](https://libisl.sourceforge.io/), which you may already have
installed (it comes with GCC).  For a non-standard install, you may need to set
the `ISL_PREFIX` environment variable.

## TODO
- dependencies for SMPT need to be clarified (Guy)
- simpler backward/forward optimizations (instead of Guy's original one)

## Working examples:
- simple_nonser (NOT serializable, terminates + TRUE)
- simple_nonser2 (NOT serializable, terminates + TRUE)
- simple_nonser3 (NOT serializable, terminates + TRUE)
- simple_nonser2_turned_ser_with_locks (always serializable, terminates + FALSE)
- state machine (always serializable, terminates + FALSE) 
- shopping cart (always serializable, terminates + FALSE)
- fred1 (always serializable, terminates + FALSE)
- fred_arith simplified until 1 (always serializable, terminates + FALSE)
- fred_arith simplified until 2 (always serializable, terminates + FALSE)
- fred2 (NOT always serializable, terminates + TRUE + counterexample)
- bank account + yields (NOT always serializable, terminates + TRUE)
- bank account + without yields (always serializable, NOT terminates)
- stateful firewall + yields (NOT always serializable, terminates + TRUE)
- stateful firewall + without yields (always serializable, terminates + FALSE)
- BGP routing (NOT serializable, terminates + TRUE)
- complex_while_with_yields (always serializable, terminates + TRUE for i<=4)
- snapshot isolation setting (medical center scenario): 2 doctors on call (always serializable, terminates + TRUE)
- snapshot isolation setting (network scenario): 2 monitoring nodes with at least one active (always serializable, terminates + TRUE)

Note: when TRUE, the answer is typically returned via BMC or K-INDUCTION
Note: when FALSE, the answer (when returned) is typically via --method --STATE-EQUATION
Note: the flag --auto-reduce speeds the SMT solver
Note: the flag --show-model returns the actual reachable marking (note that it's 
better not to use also with --auto-reduce in parallel)

Depends on SMPT.
[Add description of how to install SMPT here.]

### macOS Setup

On macOS, you'll need to install ISL and some build tools. Here's a step-by-step guide:

```bash
# Install ISL using Homebrew
brew install isl

# Install automake (needed by the isl-rs crate)
brew install automake

# Set the ISL_PREFIX environment variable (add this to your ~/.zshrc or ~/.bashrc)
export ISL_PREFIX=/opt/homebrew/Cellar/isl/0.27
```

If you're having issues with the ISL path, verify the installed version with `brew info isl` and adjust the path accordingly.

## TODO

- Add short tutorial for how to call the SMPT tool [Guy]
- Call the tool
- Extract counterexample

## Network System

Example:

    {
        "requests": [["Req1", "L0"], ["Req2", "L1"], ["Req3", "L2"]],
        "responses": [["L0", "RespA"], ["L1", "RespB"], ["L2", "RespC"]],
        "transitions": [
            ["L0", "G0", "L1", "G1"],
            ["L1", "G1", "L2", "G2"],
            ["L2", "G2", "L0", "G3"]
        ]
    }

## Syntax

### Expression Syntax

e ::=
  | n                     (constant) 
  | x := e                (local variable / packet field write)
  | x                     (read)
  | X := e                (global variable / switch variable)
  | X                     (read)
  | e + e                 (addition)
  | e - e                 (subtraction)
  | e == e                (equality check)
  | e ; e                 (sequence)
  | if(e){e}else{e}       (conditional)
  | while(e){e}           (loop)
  | yield                 (yields to the scheduler; allows other threads/packets to run)
  | exit                  (exit the entire execution of whole program / network -- maybe remove this?)
  | ?                     (nondeterministic choice between 0 and 1)
  | // text                (single-line comment, ignored by the parser)

### Multiple Requests Syntax

The parser supports multiple top-level programs with named requests:

```
request <request_name> {
    // program body
}

request <another_request_name> {
    // another program body
}
```

Examples:

```
request login {
  x := 1;
  yield;
  r := 42
}

request logout {
  y := 2;
  yield;
  r := 10
}
```

Example with arithmetic operations and comments:

```
// Main request with arithmetic operations
request main {
  x := 5 + 3;     // x = 8
  y := x - 2;     // y = 6
  z := y + y;     // z = 12
  
  // This yield allows other threads/packets to run
  yield
}
```

## VSCode Integration

This repository includes VSCode configuration for syntax highlighting of `.ser` files in the `ser-lang-vscode` directory. 

Features:
- Syntax highlighting
- Auto-closing of brackets and parentheses
- Code folding

Install with:

```bash
cd ser-lang-vscode
./build-vsix.sh
```

The script will automatically build and install the extension. You may need to restart VSCode to see the changes.
