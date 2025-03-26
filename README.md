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

## Dependencies

Depends on [isl](https://libisl.sourceforge.io/), which you may already have
installed (it comes with GCC).  For a non-standard install, you may need to set
the `ISL_PREFIX` environment variable.

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
  | e == e
  | e ; e
  | if(e){e}else{e}
  | while(e){e}
  | yield                 (yields to the scheduler; allows other threads/packets to run)
  | exit                  (exit the entire execution of whole program / network -- maybe remove this?)
  | ?                     (nondeterministic choice between 0 and 1)

### Multiple Requests Syntax

The parser now supports multiple top-level programs with named requests:

```
request <request_name> {
    // program body
}

request <another_request_name> {
    // another program body
}
```

Example:

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

The Petri net produced will have String as the request set, which comes from the request names.
