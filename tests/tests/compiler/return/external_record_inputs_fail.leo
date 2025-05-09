// Make sure we catch attempts to output external record inputs.

program test0.aleo {
    record R {
        owner: address,
        x: field,
    }

    transition foo() -> R {
        return R {
            owner: self.signer,
            x: 1field,
        };
    }
}

// --- Next Program --- //

import test0.aleo;

program test1.aleo {
    transition bar(x: test0.aleo/R) -> test0.aleo/R {
        return x;
    }

    transition baz(x: test0.aleo/R) -> test0.aleo/R {
        let y: test0.aleo/R = x;
        return y;
    }

    transition baz2(x: test0.aleo/R) -> test0.aleo/R {
        // This should be disallowed.
        let z: test0.aleo/R = x;
        let y: (test0.aleo/R, test0.aleo/R) = (z, z);
        return y.0;
    }

    transition baz3() -> test0.aleo/R {
        // But this should be OK.
        let a: test0.aleo/R = test0.aleo/foo();
        let y: (test0.aleo/R, test0.aleo/R) = (a, a);
        return y.0;
    }
}


