mod blocks;

use blocks::*;
use dwm_statusbar::*;

fn main() {
    StatusBar::new(
        " | ",
        vec![
            blocks! {
            //  { function         , update interval (ms) }
                { |_| String::new(), u64::MAX  }, // to add an extra `|` at the start
                { |_| cpu()        , 5_000     },
                { |_| battery()    , 20_000    },
                { |_| ram()        , 10_000    },
                { |_| storage()    , 60_000    },
                { |_| date()       , 30_000    },
            },
            blocks! {
                { |_| volume()     , 100       },
                { |_| brtns()      , 100       },
            },
        ],
    )
    .start();
}
