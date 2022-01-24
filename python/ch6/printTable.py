tableData = [['apples', 'oranges', 'cherries', 'banana'],
 ['Alice', 'Bob', 'Carol', 'David'], 
 ['dogs', 'cats', 'moose', 'goose']]


maxlength = 0
for row in tableData:
    if(len(row) > maxlength):
        maxlength = len(row)

def printTable():
    colWidths = [0] * len(tableData)
    for i in range(len(colWidths)):
        colWidths[i] = len(sorted(tableData[i], key=(lambda x: len(x)))[-1])

    for x in range(len(tableData[0])):
        for y in range(len(tableData)):
            print(tableData[y][x].rjust(colWidths[y]), end=' ')
        print('')
printTable()
