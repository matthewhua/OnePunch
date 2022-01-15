
#Python 程序可以调用一组基本的函数，这称为“内建函数”
# Python 也包括一组模块，称为“标准库”

# 在开始使用一个模块中的函数之前，必须用 import 语句导入该模块。在代码中，
# import 语句包含以下部分：
#  import 关键字；
#  模块的名称；
#  可选的更多模块名称，之间用逗号隔开

import random
for i in range(5):
    print(random.randint(1, 10))

# from import 语句    
# import 语句的另一种形式包括 from 关键字，之后是模块名称，import 关键字和一个星号，例如 from random import *。

# 使用这种形式的 import 语句，调用 random模块中的函数时不需要 random.前缀。
# 但是，使用完整的名称会让代码更可读，所以最好是使用普通形式的 import 语句。

import sys

while True:
    print('Type exit to exit...')
    response = input()
    if response == 'exit':
        sys.exit()
    print('You typed ' + response + '.')