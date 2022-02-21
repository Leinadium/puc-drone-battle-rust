from display import loop as display_loop
from connection import Connection

from threading import Thread


if __name__ == "__main__":
    # setup
    conn = Connection()             # connection
    t = Thread(target=conn.run)     # thread handling

    # run
    t.start()         # run connection thread
    display_loop()  # redirects to display loop

    # closing
    conn.close()    # closes the connection
    t.join()        # waits for the thread to close
