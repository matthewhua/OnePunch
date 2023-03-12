package io.matt.behavior.strategy.sort;

public class SortContext {
    private SortStrategy strategy;
    private int[] arr;
    private int N;

    public SortContext() {
    }

    public SortContext(SortStrategy strategy, int[] arr, int n) {
        this.strategy = strategy;
        this.arr = arr;
        N = n;
    }

    public void setStrategy(SortStrategy strategy) {
        this.strategy = strategy;
    }

    public void setInput(int[] iArr, int N) {
        this.arr = iArr;
        this.N = N;
    }

    public void sort() {
        this.strategy.sort(this.arr, this.N);
        System.out.print("输出:  ");
        print();
    }

    public void print() {
        for (int i = 0; i < N; i++) {
            System.out.printf("%3d", arr[i]);
        }
        System.out.println();
    }

}
