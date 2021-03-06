package backtracking;

public class EigthQueen{
    int[] result = new int[8];// 全局或成员变量, 下标表示行, 值表示 queen 存储在哪一列

    public static void main(String[] args) {
       new EigthQueen().cal8Queens(0);
    }
    public void cal8Queens(int row)
    {
        if (row == 8) {
            printQueens(result);
            return;
        }
        for (int column = 0; column < 8; column++) {// 每一行都有 8 中放法
            if (isOK(row, column)){// 有些放法不满足要求
                result[row] = column;// 第 row 行的棋子放到了 column 列
                cal8Queens(row + 1);// 考察下一行
            }
        }
    }

    private boolean isOK(int row, int column) // 判断 row 行 column 列放置是否合适
    {
        int leftUp = column - 1, rightUp = column + 1;
        for (int i = row - 1; i >= 0 ; --i) {// 逐行往上考察每一行
            if (result[i] == column) return false; // 第 i 行的 column 列有棋子吗？
            if (leftUp >= 0)//考察左对上角: 第 i 行的column 列有棋子吗??
            {
                if (result[i] == leftUp) return false;
            }
           if (rightUp < 8) //考察右上对角线：第 i 行 rightup 列有棋子吗？
               if (result[i] == rightUp) return false;
           --leftUp; ++rightUp;
        }
        return true;
    }

    private void printQueens(int[] result) // 打印出一个二维矩阵
    {
        for (int row = 0; row < 8; row++) {
            for (int column = 0; column < 8; column++) {
                if (result[row] == column) System.out.print("Q ");
                else System.out.print("* ");
            }
            System.out.println();
        }
        System.out.println();
    }
}

