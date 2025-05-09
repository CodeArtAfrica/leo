/*
seed = 123456789

[case]
program = "test1.aleo"
function = "check_io"
input = []
[case]
program = "test1.aleo"
function = "foo"
input = ["0group"]
*/

program test0.aleo {
    record R {
        owner: address,
        x: field,
    }

    // As of commit 314d56a9e3626418e8a953ecb81f595555716459, this would fail
    // as we're outputting a record we just received as input. Now we should
    // clone the record (by casting it into a new one).
    transition input_output(r: R) -> (u8, R) {
        return (1u8, r);
    }

    transition make() -> R {
        return R {
            owner: self.signer,
            x: 0field,
        };
    }
}

// --- Next Program --- //

import test0.aleo;

program test1.aleo {
    transition check_io() -> test0.aleo/R {
        let r: test0.aleo/R = test0.aleo/make();
        return test0.aleo/input_output(r).1;
    }

    struct S {
        x: field,
        y: field,
    }

    // As of commit 314d56a9e3626418e8a953ecb81f595555716459,
    // programs with duplicate outputs did not work.
    // Now they should clone the duplicates to satisfy SnarkVM.
    transition foo(y: group) -> (S, S, group, group, field, field, u8, u8) {
        let x: S = S {
            x: 1field,
            y: 2field,
        };
        return (x, x, y, y, 2field, 2field, 1u8, 1u8);
    }
}

