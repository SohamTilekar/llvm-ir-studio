from LSP.plugin import LspPlugin
from LSP.plugin import register_plugin
from LSP.plugin import unregister_plugin


class LlvmIr(LspPlugin):
    @classmethod
    def name(cls) -> str:
        return "llvm-ir-ls"


def plugin_loaded() -> None:
    register_plugin(LlvmIr)


def plugin_unloaded() -> None:
    unregister_plugin(LlvmIr)
