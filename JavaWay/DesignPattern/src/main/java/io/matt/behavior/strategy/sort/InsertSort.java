package io.matt.behavior.strategy.sort;

public class InsertSort implements SortStrategy{

    public InsertSort() {
        System.out.println("插入排序");
    }

    @Override
    public void sort(int[] arr, int N) {
        int i, j;
        for (i = 1; i < N; i++) {
            for (j = i - 1; j >= 0; j--) {
                if (arr[i] > arr[j]) {
                    break;
                }
            }
            int temp = arr[i];
            for (int k = i - 1; k > j; k--) {
                arr[k + 1] = arr[k];
            }
            arr[j + 1] = temp;
        }
    }
}
