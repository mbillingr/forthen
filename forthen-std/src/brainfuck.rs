use forthen_core::errors::*;
use forthen_core::State;

/// Load basic operations into the dictionary
pub fn brainfuck(state: &mut State) -> Result<()> {
    state.run(
        "
        MODULE brainfuck

        USE :std:

        (limitations:
            - no , command for input
            - individual commands must be separated by spaces because they are implemented as words,
              or they must be preceded by the BF parse word
            - storing tape in a table because we lack arrays
            - need one more word to use quotations as [ ] loops )

        : init (-- tape pos)
            0 list-make
            30000 [ 0 push-back ] repeat
            0
        ;

        :: query   (tape pos -- tape pos x)   set pos get pos list-get get pos swap ;
        :: update   (tape pos x -- tape' pos)   set x set pos get pos get x list-set get pos ;

        : > (tape pos -- tape pos') 1 + ;
        : < (tape pos -- tape pos') 1 - ;
        : + (tape pos -- tape' pos) query 1 + update ;
        : - (tape pos -- tape' pos) query 1 - update ;
        : . (tape pos -- tape pos') query emit ;

        :: loop   (tape pos -- tape' pos')   set inner [ query 0 != ] [ get inner call ] while ;

        : insert-cmd (ops cmds current -- ops' cmds) lookup rot swap bake swap ;

        SYNTAX: compare dup next_token == ;

        SYNTAX: BF
            next_token str-to-list
            [ list-empty? not ]
            [
                pop-front
                [
                    [ [ dup \"<\" == ] [ insert-cmd ] ]
                    [ [ dup \">\" == ] [ insert-cmd ] ]
                    [ [ dup \"+\" == ] [ insert-cmd ] ]
                    [ [ dup \"-\" == ] [ insert-cmd ] ]
                    [ [ dup \".\" == ] [ insert-cmd ] ]
                ] cond
            ] while
            drop
        ;


        (Brainfuck Example)

        : hello   (tape pos -- tape' pos')
            + + + + + + + + + +
            [
               BF >+++++++>++++++++++>+++>+<<<<-
            ] loop
            BF >++.                (H)
            BF >+.                 (e)
            BF +++++++..           (ll)
            BF +++.                (o)
            BF >++.                ( )
            BF <<+++++++++++++++.  (W)
            BF >.                  (o)
            BF +++.                (r)
            BF ------.             (l)
            BF --------.           (d)
            BF >+.                 (!)
            BF >.                  (LF)
            BF +++.                (CR)
        ;

        init hello
        drop drop

        END-MODULE
    ",
    )
}
