# 一些代码行可以作为一组，放在“代码块”中。可以根据代码行的缩进，知道
# 代码块的开始和结束。代码块有 3 条规则。
# 1．缩进增加时，代码块开始。
# 2．代码块可以包含其他代码块。
# 3．缩进减少为零，或减少为外面包围代码块的缩进，代码块就结束了。
# 看一些有缩进的代码，更容易理解代码块。所以让我们在一小段游戏程序中，

name = 'Matthew'
password = 'sword'

# 将在语句的条件为 True 时执行。如果条件为 False，子句将跳过。
# 在英文中，if 语句念起来可能是：“如果条件为真，执行子句中的代码。”在 Python
# 中，if 语句包含以下部分：  if 关键字；
#  条件（即求值为 True 或 False 的表达式）；
#  冒号；
#  在下一行开始，缩进的代码块（称为 if 子句）。

if name == 'Marry':
    print('Hello Marry')
if password == 'sword':  
    print('Access granted')
else:
    print('Wrong password')

age = 11

if name == 'Alice':
    print('Hi Alice')
elif age < 12:
     print('You are not Alice, kiddo.')
else:
 print('You are neither Alice nor a little kid.')