package io.jmhDemo

import org.openjdk.jmh.annotations.*
import org.openjdk.jmh.runner.Runner
import org.openjdk.jmh.runner.options.OptionsBuilder
import java.util.concurrent.TimeUnit

/**
 * 对比 Sequence 和 List
 *
 *  Kotlin 1.2.70 的 release note
 * 使用 Sequence 有助于避免不必要的临时分配开销，并且可以显着提高复杂处理 PipeLines 的性能。
 */

@BenchmarkMode(Mode.Throughput) // 基准测试的模式，采用整体吞吐量的模式
@Warmup(iterations = 3) // 预热次数
@Measurement(iterations = 10, time = 5, timeUnit = TimeUnit.SECONDS) // 测试参数，iterations = 10 表示进行10轮测试
@Threads(8) // 每个进程中的测试线程数
@Fork(2)  // 进行 fork 的次数，表示 JMH 会 fork 出两个进程来进行测试
@OutputTimeUnit(TimeUnit.MILLISECONDS) // 基准测试结果的时间类型
open class SequenceBenchmark {

    @Benchmark
    fun testSequence(): Int{
        return sequenceOf(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
            .map { it * 2 }
            .filter{ it % 3 == 0}
            .map { it + 1}
            .sum()
    }


    @Benchmark
    fun testList():Int {

        return listOf(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
            .map { it * 2 }
            .filter { it % 3 == 0 }.sumOf { it + 1 }
    }

}

fun main(){
    val options = OptionsBuilder()
        .include(SequenceBenchmark::class.java.simpleName)
        .output("benchmark_sequence.log")
        .build()
    Runner(options).run()
}