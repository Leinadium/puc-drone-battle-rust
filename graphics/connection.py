from paho.mqtt.client import Client, MQTTMessage
from threading import Lock
import json

TOPICO = 'puc-drone-battle-rust'


class Data:
    object = None

    def __init__(self):
        self.lock: Lock = Lock()
        self.field: dict = {}
        self.drone: dict = {}
        self.update: bool = False

    def from_dict(self, d: dict):
        with self.lock:
            ident = d.get('id')
            # field
            if 'field' in d:
                self.field[ident] = d.get('field')
            if 'bot' in d:
                self.drone[ident] = d.get('bot')
            self.update = True
        return

    def has_to_update(self) -> bool:
        with self.lock:
            if self.update:
                self.update = False
                return True
        return False

    def block(self):
        self.lock.acquire(blocking=True, timeout=-1)

    def unblock(self):
        if self.lock.locked():
            self.lock.release()

    # factory
    @classmethod
    def get_data(cls):
        if cls.object is None:
            cls.object = Data()
        return cls.object


def _on_connect(client: Client, _userdata, _flags, rc):
    print(f"[CONNECTION] connected with code {rc}")
    client.subscribe(TOPICO)
    print(f"[CONNECTION] subscribed to topic {TOPICO}")
    return


def _on_message(_client: Client, _userdata, msg: MQTTMessage):
    try:
        json_data: dict = json.loads(msg.payload)
        Data.get_data().from_dict(json_data)
        # print("[CONNECTION] received data: ", json_data)

    except json.JSONDecodeError:
        print("[CONNECTION] error while parsing data")
    return


class Connection:
    def __init__(self):
        self.c = Client()
        self.c.on_connect = _on_connect
        self.c.on_message = _on_message

    def run(self):
        print("[CONNECTION] Connecting to localhost:1883")
        self.c.connect("localhost", 1883, 60)
        self.c.loop_forever()

    def close(self):
        print("[CONNECTION] closing")
        self.c.disconnect()
