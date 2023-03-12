package io.matt.behavior.strategy.sort;

// 具体策略：冒泡排序
public class BubbleSort implements SortStrategy {

    public BubbleSort() {
        System.out.println("冒泡排序");
    }

    @Override
    public void sort(int[] arr, int N) {
        for (int i = 0; i < N; i++) {
            for (int l = 0; l < N - i - 1; l++) {
                if (arr[l] > arr[l + 1]) {
                    int tmp = arr[l];
                    arr[l] = arr[l + 1];
                    arr[l + 1] = tmp;
                }
            }
        }
    }
}
