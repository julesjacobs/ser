# Ser

Serializability checker.

## TODO

- Compute regex
- Compute whatever input the tool takes for the semilinear set

- Compute automaton from parsed program
- Compute petri net from automaton

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