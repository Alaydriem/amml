# Decompiles one function. Arguments: <function start RVA in hex> <output file>.
# Runs as a Ghidra headless post-script (Jython). The program needs no analysis:
# the script disassembles from the start and creates the function when it is missing.
from ghidra.app.decompiler import DecompInterface
from ghidra.app.cmd.disassemble import DisassembleCommand
from ghidra.app.cmd.function import CreateFunctionCmd
from ghidra.util.task import ConsoleTaskMonitor

TIMEOUT_SECONDS = 120

args = getScriptArgs()
if len(args) != 2:
    printerr("usage: DecompileAt.py <rva> <out>")
    exit()

rva = int(args[0], 16)
out_path = args[1]
start = currentProgram.getImageBase().add(rva)
monitor = ConsoleTaskMonitor()

if currentProgram.getListing().getInstructionAt(start) is None:
    DisassembleCommand(start, None, True).applyTo(currentProgram, monitor)

function = getFunctionAt(start)
if function is None:
    CreateFunctionCmd(start).applyTo(currentProgram, monitor)
    function = getFunctionAt(start)
if function is None:
    printerr("no function could be made at " + str(start))
    exit()

decompiler = DecompInterface()
decompiler.openProgram(currentProgram)
result = decompiler.decompileFunction(function, TIMEOUT_SECONDS, monitor)
if not result.decompileCompleted():
    printerr("decompile failed: " + str(result.getErrorMessage()))
    decompiler.dispose()
    exit()

text = result.getDecompiledFunction().getC()
decompiler.dispose()
handle = open(out_path, "wb")
handle.write(text.replace(chr(13), "").encode("utf-8"))
handle.close()
print("DecompileAt: wrote " + str(len(text)) + " characters for " + function.getName() + " at " + str(start))
