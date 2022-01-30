# Batalha de Drones

Projeto para a 1ª Competição de IA do Departamento de Informática da PUC-Rio

Este é uma cópia do [trabalho 4 para INF1771 - Inteligencia Artificial](https://github.com/Leinadium/puc-drone-battle), 
porém dessa vez este trabalho foi escrito em Rust, em vez de Java.

Desenvolvido por Daniel Guimarães

## Descrição

A descrição da competição pode ser encontrada [aqui](https://augustobaffa.pro.br/site/desafios-online/inf1771-inteligencia-artificial-desafio-dos-drones/)

Este projeto envolve a construção de uma inteligência artificial para competir numa batalha de “drones”.

## Funcionamento

A inteligência artificial utilizada foi uma *máquina de estados* , com usos de algoritmos de *pathfinding*

A cada tick (100ms), é feita uma observação em volta do drone. A próxima decisão deve ser tomada em base dessas observações,
assim como outros detalhes, como o mapa em volta.

Os estados estão abaixo, em ordem decrescente de relevância:

* **Atacar** : Ataca o inimigo que encotrou

* **Fugir** : Tenta fugir dos disparos inimigos

* **Recarregar** : Tenta recarregar a sua energia o mais rápido possível

* **Coletar** : Coleta o ouro

* **Explorar** : Explora o mapa de acordo com os algoritmos criados


## Árvore de Decisões para os Estados

Para poder decidir qual estado utilizar de maneira eficiente, foi utilizado uma árvore de decisões com a seguinte estrutura:

```text
|
|- se tiver ouro no meu bloco -> COLETAR
|- se tiver powerup no meu bloco e estiver com menos de 100 de energia -> RECARREGAR
|- se em processo de fuga -> FUGIR
|- se tiver inimigo na frente, não ter disparado 10 vezes seguidas, e energia maior que 30
|    e nao tiver uma parede entre eu e o inimigo na frente -> ATACAR
|- se tomei dano e não tiver inimigo na frente -> FUGIR
|- se tiver inimigo na frente ou nos lados, e estiver com menos de 30 de energia -> FUGIR
|- se energia menor que 80 -> RECARREGAR
|- se tiver ouro para coletar -> COLETAR
|-- se nenhum dos anteriores -> EXPLORAR
```

## Funcionamento de cada um dos estados

A seguir estão os funcionamentos de cada um dos estados possíveis do drone

### ATACAR

O drone irá atirar. Note que essa ação só se repete no máximo dez vezes seguidas, para evitar
que o drone fique preso atacando algum alvo falso.

### FUGIR

O bloco que o drone tomou dano, assim como as adjacências são marcadas como blocos *inseguros* , que são penalizados
no algorimo de pathfinding.

Além disso, caso o drone tenha recebido um dano ou ter um inimigo na sua frente, ele tenta encontrar uma caminho para
algum bloco nas suas laterais para sair do campo de visão. Os blocos consultados são os seguintes:

```text
00...00 
00...00
00.x.00  onde o x é o drone, e 0 são os blocos procurados
00...00
00...00
```

Caso só haja um inimigo por perto, o drone irá virar para a esquerda, tentando encontrar o inimigo.

Caso não haja caminho possível ou algo de errado ocorrer no algoritmo, a ação executada será de atacar.

### COLETAR

Se tiver um tesouro na posição atual, a ação deve ser de pegar o ouro.

Se não, pega o caminho para o tesouro mais próximo e perto de renascer, e segue. Se ele já estiver seguindo um outro caminho
para outro tesouro, continua seguindo ele (para evitar ficar indo e voltando).

Se houver algum erro, não fazer nada por um tick.

### RECARREGAR

As ações de recarregar são semelhantes ao *coletar* quando há um powerup pronto para coletar.

Caso não tenha um powerup pronto, o drone irá explorar em volta da posição até ele estiver pronto para ser coletado.

### EXPLORAR

Entre todos os blocos seguros ainda não explorados, é escolhido que o menor custo, em que o custo pode ser calculado pela
fórmula:

```distanciaEuclidiana(bloco, pontoFocal) * 2 + distanciaVerdadeira(bloco, drone)```

em que *pontoFocal* é o bloco de spawn do drone, ou o ponto médio dos ouros conhecidos, e a distância verdadeira é a quantidade
de movimentos que o drone deve fazer para chegar naquele bloco.

## Pathfinding

O algoritmo de pathfinding foi o responsável de fazer toda a movimentação do drone, em todos os estados. Seja ao explorar,
fugir ou recarregar. Tanto o ataque (atirar) como a fuga de algum inimigo por perto (girar para a esquerda) eram as únicas
ações que o pathfinding não era responsável.

O algoritmo de *pathfinding* é o [A*](https://en.wikipedia.org/wiki/A*_search_algorithm), com algumas alterações:

* O algoritmo não utiliza as posições no mapa, mas sim a combinação de (x, y, direção). Logo, o drone na mesma posição, mas
  em direções diferentes são vértices diferentes no grafo do algoritmo.

* Todos os vértices possuem a mesma distância dos seus vizinhos (andar para frente, para trás, e virar para os lados), 1;

* Um vértice que esteja num lugar seguro ainda não explorado tem um custo de 0.8;

* Um vértice que tenha sido resultado de andar para trás tem custo de 1.5;

* Um vértice em uma posição insegura (em que o drone acabou de sofrer um dano) possui um custo de 10.


## Requirementos

