use forthen_core::errors::*;
use forthen_core::State;

/// Load basic operations into the dictionary
pub fn brainfuck(state: &mut State) -> Result<()> {
    state.run(
        "
        MODULE brainfuck

        USE :ops:
        USE :scope:
        USE :stack:
        USE :table:

        MODULE utils
            : stash dup rot swap ;
            :: peek set c set b set a get a get b get c get a ;

            (todo: it would be nice to hide the internals (callback, loop quotation) from the callee)
            : for
                swap
                [
                    1 -
                    peek call
                    dup 0 ==
                    [ drop drop drop ]
                    [
                        swap stash call
                    ]
                    if
                ]
                stash call
            ;

            :: while
                set callee
                set cond
                [
                    get cond call
                    [
                        get callee call
                        get loop call
                    ]
                    [ ]
                    if
                ]
                set loop
                get loop call
            ;

            :: init_cell
                set idx
                set loop
                set callback

                get idx repr 0 set_attribute

                get callback
                get loop
                get idx
            ;
        END-MODULE

        USE utils:for
        USE utils:while
        USE utils:init_cell

        (limitations:
            - no , command for input
            - individual commands must be separated by spaces because they are implemented as words
            - storing tape in a table because we lack arrays
            - loops cause stack overflows, so we can't initialize more than 100 cells)

        : init
            {}
            100 [ init_cell ] for
            0
        ;

        : query dup repr rot swap get_attribute rot rot swap rot ;
        :: update swap dup repr swap set ptr swap set_attribute get ptr ;

        : > 1 + ;
        : < 1 - ;
        : + query 1 + update ;
        : - query 1 - update ;
        : . query . ;

        :: loop set inner [ query 0 != ] [ get inner call ] while ;


        END-MODULE
    ",
    )
}
