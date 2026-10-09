use std::path::Path;

use crate::toolchain::python::path_string;
use crate::toolchain::shell_activation::{
    ShellActivation, fish_quote, posix_quote, powershell_quote,
};

pub(super) fn activation(conda: &Path, prefix: &Path) -> ShellActivation {
    let conda = path_string(conda);
    let prefix = path_string(prefix);
    ShellActivation {
        posix: format!(
            "eval \"$({} shell.posix hook)\" && conda activate {}",
            posix_quote(&conda),
            posix_quote(&prefix)
        ),
        fish: format!(
            "{} shell.fish hook | source; and conda activate {}",
            fish_quote(&conda),
            fish_quote(&prefix)
        ),
        powershell: format!(
            "(& {} shell.powershell hook) | Out-String | Invoke-Expression; conda activate {}",
            powershell_quote(&conda),
            powershell_quote(&prefix)
        ),
    }
}
