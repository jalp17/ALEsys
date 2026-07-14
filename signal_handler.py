import signal
import sys

def signal_handler(sig, frame):
    logger.info("Interrupción recibida, limpiando...")
    self.close()
    sys.exit(0)

signal.signal(signal.SIGINT, signal_handler)