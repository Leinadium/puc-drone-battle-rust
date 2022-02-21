import pygame
from pygame import Color
from random import randint
from typing import Dict

from connection import Data

SCREEN_SIZE = SCREEN_WIDTH, SCREEN_HEIGHT = 1200, 700
BLOCK_SIZE = 20
HORIZONTAL_BORDER = (SCREEN_WIDTH - BLOCK_SIZE * 59) // 2
VERTICAL_BORDER = (SCREEN_HEIGHT - BLOCK_SIZE * 34) // 2

COLOR_GREEN = Color(0, 200, 0)
COLOR_BLUE = Color(100, 100, 255)
COLOR_YELLOW = Color(255, 255, 50)
COLOR_RED = Color(200, 0, 0)
COLOR_WHITE = Color(240, 240, 240)
COLOR_GRAY = Color(50, 50, 50)
COLOR_BLACK = Color(0, 0, 0)
COLOR_ORANGE = Color(255, 100, 0)


class PlayerColor:
    def __init__(self):
        self.d: Dict[str, Color] = {}

    def get(self, ident: str) -> Color:
        if ident not in self.d:
            self.d[ident] = Color(randint(0, 255), randint(0, 255), randint(0, 255))
        return self.d[ident]


PLAYER_COLOR = PlayerColor()


def coords_to_pos(x: int, y: int):
    return HORIZONTAL_BORDER + BLOCK_SIZE * x, VERTICAL_BORDER + BLOCK_SIZE * y


def type_to_color(t: str) -> Color:
    if t == "SAFE":
        return COLOR_GREEN
    elif t == "EMPTY":
        return COLOR_WHITE
    elif t == "DANGER":
        return COLOR_RED
    elif t == "UNKNOWN":
        return COLOR_GRAY
    elif t == "WALL":
        return COLOR_BLACK
    elif t == "GOLD":
        return COLOR_YELLOW
    elif t == "POWERUP":
        return COLOR_BLUE
    else:
        return COLOR_WHITE


def check_close() -> bool:
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            return True
    return False


def coords_to_id(x, y) -> int:
    return x * 100 + y


def id_to_cords(coord_id):
    return coord_id // 100, coord_id % 100


def generate_full_unknown_map() -> dict:
    return {coords_to_id(x, y): type_to_color("UNKNOWN") for x in range(59) for y in range(34)}


def generate_map(d: Data) -> dict:
    ret = generate_full_unknown_map()
    try:
        for field in d.field.values():
            for position in field.get('map'):
                x, y, t = int(position[0]), int(position[1]), position[2]
                coords = coords_to_id(x, y)

                if ret.get(coords) == COLOR_WHITE and t == "SAFE":
                    continue    # superposicao de verde no branco

                ret[coords_to_id(x, y)] = type_to_color(t)

            midpoint = field.get('midpoint')
            ret[coords_to_id(int(midpoint[0]), int(midpoint[1]))] = COLOR_ORANGE

    except Exception as e:
        print("[DISPLAY]: invalid map: ", e)

    return ret


def generate_map_info(d: Data) -> dict:
    ret = {}
    try:
        for field in d.field.values():
            for a in ('gold', 'powerup'):
                for gold in field.get(a):
                    x, y, t = int(gold[0]), int(gold[1]), int(gold[2])
                    ret[coords_to_id(x, y)] = f"{t // 1000}s"
    except Exception as e:
        print("[DISPLAY]: invalid map info: ", e)

    return ret


def generate_player(d: Data) -> dict:
    ret = {}
    try:
        for ident, player in d.drone.items():
            x, y = player.get('x'), player.get('y')
            ret[coords_to_id(x, y)] = (
                ident, player.get('energy'),
                player.get('score'), player.get('dir'),
                player.get('state')
            )

    except Exception as e:
        print("[DISPLAY]: invalid player info: ", e)

    return ret


def generate_path(d: Data) -> dict:
    ret = {}
    try:
        ret = {ident: f.get('current_path') for ident, f in d.field.items()}

    except Exception as e:
        print("[DISPLAY]: invalid path info: ", e)

    return ret


def draw_map(screen: pygame.Surface, map_info: dict):
    rect = pygame.rect.Rect(0, 0, BLOCK_SIZE, BLOCK_SIZE)
    try:
        for coords_id, color in map_info.items():
            rect.x, rect.y = coords_to_pos(*id_to_cords(coords_id))
            pygame.draw.rect(screen, color, rect, width=0, border_radius=1)

    except Exception as e:
        print("[DISPLAY]: error drawing map: ", e)


def draw_map_info(screen: pygame.Surface, map_info: dict, font: pygame.font.Font):
    try:
        for coords_id, text_string in map_info.items():
            text = font.render(text_string, True, COLOR_BLACK)
            rect = text.get_rect()
            x, y = coords_to_pos(*id_to_cords(coords_id))
            rect.x = x - (text.get_size()[0] - BLOCK_SIZE) // 2
            rect.y = y
            screen.blit(text, rect)

    except Exception as e:
        print("[DISPLAY]: error drawing map info: ", e)


def draw_player(screen: pygame.Surface, player_info: dict, font: pygame.font.Font):
    try:
        for coords_id, player in player_info.items():
            x, y = coords_to_pos(*id_to_cords(coords_id))
            # player
            rect = pygame.rect.Rect(x + 1, y + 1, BLOCK_SIZE - 1, BLOCK_SIZE - 1)
            pygame.draw.rect(screen, PLAYER_COLOR.get(player[0]), rect, width=0, border_radius=3)

            # direction
            player_center = (x + BLOCK_SIZE // 2, y + BLOCK_SIZE // 2)
            if player[3] == "SOUTH":
                direction_point = (player_center[0], player_center[1] + BLOCK_SIZE)
            elif player[3] == "EAST":
                direction_point = (player_center[0] + BLOCK_SIZE, player_center[1])
            elif player[3] == "WEST":
                direction_point = (player_center[0] - BLOCK_SIZE, player_center[1])
            else:
                direction_point = (player_center[0], player_center[1] - BLOCK_SIZE)
            pygame.draw.line(screen, COLOR_BLACK, player_center, direction_point, width=4)

            # nome
            text_name = font.render(f"{player[0]}", True, COLOR_BLACK)
            rect_name = text_name.get_rect()
            rect_name.x, rect_name.y = x - text_name.get_size()[0], y
            screen.blit(text_name, rect_name)

            # energia
            text_energy = font.render(f"e: {player[1]}", True, COLOR_BLUE)
            rect_energy = text_energy.get_rect()
            rect_energy.x, rect_energy.y = x - (text_energy.get_size()[0] - BLOCK_SIZE) // 2, y - BLOCK_SIZE
            screen.blit(text_energy, rect_energy)

            # score
            text_score = font.render(f"s: {player[2]}", True, COLOR_ORANGE)
            rect_score = text_score.get_rect()
            rect_score.x, rect_score.y = x - (text_score.get_size()[0] - BLOCK_SIZE) // 2, y + BLOCK_SIZE
            screen.blit(text_score, rect_score)

            # state
            text_state = font.render(f"{player[4]}", True, COLOR_RED)
            rect_state = text_state.get_rect()
            rect_state.x, rect_state.y = x + BLOCK_SIZE, y
            screen.blit(text_state, rect_state)

    except Exception as e:
        print("[DISPLAY]: error drawing player info: ", e)


def draw_paths(screen: pygame.Surface, paths_info: dict):
    hbs = BLOCK_SIZE // 2       # HALF_BLOCK_SIZE
    radius = BLOCK_SIZE // 3
    try:
        for ident, path in paths_info.items():
            color = PLAYER_COLOR.get(ident)
            if len(path) < 2:
                continue
            for i, (x, y) in enumerate(path[1:]):
                x, y = coords_to_pos(x, y)
                pygame.draw.circle(screen, color, (x + hbs, y + hbs), radius)
                # pygame.draw.line(screen, color, (x0 + hbs, y0 + hbs), (x1 + hbs, y1 + hbs), width=3)

    except Exception as e:
        print("[DISPLAY]: error drawing paths: ", e)


def draw_all(screen: pygame.Surface, data: Data, font: pygame.font.Font):
    data.block()
    gen_map = generate_map(data)
    gen_map_info = generate_map_info(data)
    gen_player = generate_player(data)
    gen_path = generate_path(data)
    data.unblock()

    screen.fill(COLOR_BLACK)
    draw_map(screen, gen_map)
    draw_map_info(screen, gen_map_info, font)
    draw_paths(screen, gen_path)
    draw_player(screen, gen_player, font)
    pygame.display.flip()


def loop():
    print("[DISPLAY] starting loop")

    pygame.init()
    screen = pygame.display.set_mode(SCREEN_SIZE)
    pygame.display.set_caption("puc-drone-battle-rust")

    font_text = pygame.font.SysFont('bahnschrift.ttf', 20, bold=False)

    while not check_close():
        game_data = Data.get_data()
        if game_data.has_to_update():
            draw_all(screen, game_data, font_text)

        pygame.time.wait(50)

    pygame.quit()
