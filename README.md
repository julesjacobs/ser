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

## TODO

- Compute regex
- Compute whatever input the tool takes for the semilinear set

- Compute automaton from parsed program [Guy]
- Compute petri net from automaton

- Call the tool
- Extract counterexample
- Add short tutorial for how to call the SMPT tool [Guy]

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

e ::=
  | x := e
  | X := e
  | e == e
  | e ; e
  | if(e){e}else{e}
  | while(e){e}
  | yield
  | exit
  | ?
  | n
  | x
  | X