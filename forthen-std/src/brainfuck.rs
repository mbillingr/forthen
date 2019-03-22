use forthen_core::errors::*;
use forthen_core::State;

/// Load basic operations into the dictionary
pub fn brainfuck(state: &mut State) -> Result<()> {
    state.run(
        "
        MODULE brainfuck

        USE :loop:
        USE :ops:
        USE :scope:
        USE :stack:
        USE :table:

        (limitations:
            - no , command for input
            - individual commands must be separated by spaces because they are implemented as words
            - storing tape in a table because we lack arrays
            - need one more word to use quotations as [ ] loops )

        : init (-- tape pos)
            {}
            0 30000 [ repr 0 set_attribute ] for
            0
        ;

        : query   (tape pos -- tape pos x)   dup repr rot swap get_attribute rot rot swap rot ;
        :: update   (tape pos x -- tape' pos)   swap dup repr swap set ptr swap set_attribute get ptr ;

        : > (tape pos -- tape pos') 1 + ;
        : < (tape pos -- tape pos') 1 - ;
        : + (tape pos -- tape' pos) query 1 + update ;
        : - (tape pos -- tape' pos) query 1 - update ;
        : . (tape pos -- tape pos') query emit ;

        :: loop   (tape pos -- tape' pos')   set inner [ query 0 != ] [ get inner call ] while ;


        (Brainfuck Example)

        init
        + + + + + + + + + +
        [
          > + + + + + + + > + + + + + + + + + + > + + + > + < < < < -
        ] loop
        > + + .                              (H)
        > + .                                (e)
        + + + + + + + . .                    (ll)
        + + + .                              (o)
        > + + .                              ( )
        < < + + + + + + + + + + + + + + + .  (W)
        > .                                  (o)
        + + + .                              (r)
        - - - - - - .                        (l)
        - - - - - - - - .                    (d)
        > + .                                (!)
        > .                                  (LF)
        + + + .                              (CR)
        drop drop

        END-MODULE
    ",
    )
}