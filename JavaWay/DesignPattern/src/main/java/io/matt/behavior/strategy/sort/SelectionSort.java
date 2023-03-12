package io.matt.behavior.strategy.sort;

// 具体策略：选择排序
public class SelectionSort implements SortStrategy{


    public SelectionSort() {
        System.out.println("选择排序\n");
    }

    @Override
    public void sort(int[] arr, int N) {
        int i, j, k;
        for (i = 0; i < N; i++) {
            k = i;
            for (j = i + 1; j < N; j++) {
                if (arr[j] < arr[k]) {
                    k = j;
                }
            }
            int temp = arr[i];
            arr[i] = arr[k];
            arr[k] = temp;
        }
    }
}
