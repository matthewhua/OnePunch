package backtracking;

public class EigthQueen{
    int[] result = new int[8];// ȫ�ֻ��Ա����, �±��ʾ��, ֵ��ʾ queen �洢����һ��

    public static void main(String[] args) {
       new EigthQueen().cal8Queens(0);
    }
    public void cal8Queens(int row)
    {
        if (row == 8) {
            printQueens(result);
            return;
        }
        for (int column = 0; column < 8; column++) {// ÿһ�ж��� 8 �зŷ�
            if (isOK(row, column)){// ��Щ�ŷ�������Ҫ��
                result[row] = column;// �� row �е����ӷŵ��� column ��
                cal8Queens(row + 1);// ������һ��
            }
        }
    }

    private boolean isOK(int row, int column) // �ж� row �� column �з����Ƿ����
    {
        int leftUp = column - 1, rightUp = column + 1;
        for (int i = row - 1; i >= 0 ; --i) {// �������Ͽ���ÿһ��
            if (result[i] == column) return false; // �� i �е� column ����������
            if (leftUp >= 0)//��������Ͻ�: �� i �е�column ����������??
            {
                if (result[i] == leftUp) return false;
            }
           if (rightUp < 8) //�������϶Խ��ߣ��� i �� rightup ����������
               if (result[i] == rightUp) return false;
           --leftUp; ++rightUp;
        }
        return true;
    }

    private void printQueens(int[] result) // ��ӡ��һ����ά����
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

