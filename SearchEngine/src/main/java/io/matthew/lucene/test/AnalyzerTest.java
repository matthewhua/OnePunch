package io.matthew.lucene.test;

import io.matthew.lucene.bean.Book;
import org.apache.lucene.analysis.Analyzer;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.*;
import org.apache.lucene.index.IndexWriter;
import org.apache.lucene.index.IndexWriterConfig;
import org.apache.lucene.store.FSDirectory;
import org.junit.Test;
import org.wltea.analyzer.lucene.IKAnalyzer;

import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;

public class AnalyzerTest {


    // 中文分词器
    @Test
    public void testCreateIndex() throws Exception {

        // 1. 采集数据 和之前完全相同
        final ArrayList<Book> books = new ArrayList<>();
        final Book build = Book.newBuilder().setId(1).setPrice(100.456f).setName("Matthew2").setDesc("你是爆狼吗爆").build();
        books.add(build);
        // 2. 创建Document文档对象
        List<Document> documents = new ArrayList<>();
        for (Book book : books) {
            final Document document = new Document();
            // IntPoint 分词 索引 不存储 存储组合 StoredFied
            final IntPoint id = new IntPoint("id", book.getId());
            final StoredField id_v = new StoredField("id", book.getId());
            // 分词、索引、存储 TextField
            final TextField name = new TextField("name", book.getName(), Field.Store.YES);
            // 分词、索引、不存储 但是是数字类型，所以使用FloatPoint
            final FloatPoint price = new FloatPoint("price", book.getPrice());
            // 分词、索引、存储 TextField 为了看到分词效果设置成存储
            final TextField desc = new TextField("desc", book.getDesc(), Field.Store.YES);

            // 将field域设置到Document对象中
            document.add(id);
            document.add(id_v);
            document.add(name);
            document.add(price);
            document.add(desc);

            documents.add(document);
        }

        //3.创建StandardAnalyzer 分词器 对文档进行分词  把 StandardAnalyzer 变成 IKAnalyer

        Analyzer analyzer  = new IKAnalyzer(); //中文分词
        //final StandardAnalyzer standardAnalyzer = new StandardAnalyzer(); // 标准分词器
        // 创建Directory  哦[和 IndexWriterConfig 对象
        final FSDirectory directory = FSDirectory.open(Paths.get("D:/lucene/index5"));
        final IndexWriterConfig indexWriterConfig = new IndexWriterConfig(analyzer);

        // 4. 创建IndexWriter 写入对象
        final IndexWriter indexWriter = new IndexWriter(directory, indexWriterConfig);
        // 添加文档对象
        for (Document doc : documents) {
            indexWriter.addDocument(doc);
        }
        // 释放资源
        indexWriter.close();
    }
}
