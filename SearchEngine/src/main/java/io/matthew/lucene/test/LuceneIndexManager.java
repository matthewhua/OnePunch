package io.matthew.lucene.test;

import org.apache.lucene.analysis.Analyzer;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.Document;
import org.apache.lucene.document.Field;
import org.apache.lucene.document.TextField;
import org.apache.lucene.index.IndexWriter;
import org.apache.lucene.index.IndexWriterConfig;
import org.apache.lucene.index.Term;
import org.apache.lucene.store.Directory;
import org.apache.lucene.store.FSDirectory;
import org.junit.Test;

import java.nio.file.Paths;

public class LuceneIndexManager {


    /*** 索引添加*/
    @Test
    public void indexCreate() throws Exception {
        // 创建分词器
        final StandardAnalyzer analyzer = new StandardAnalyzer();
        // 创建Directory 流对象
        final FSDirectory directory = FSDirectory.open(Paths.get("D:/lucene/index3"));
        final IndexWriterConfig config = new IndexWriterConfig(analyzer);
        // 创建索引写入对象
        final IndexWriter indexWriter = new IndexWriter(directory, config);
        // 创建Document
        final Document document = new Document();
        document.add(new TextField("id", "1001", Field.Store.YES));
        document.add(new TextField("name", "game", Field.Store.YES));
        document.add(new TextField("desc", "one world one dream", Field.Store.NO));
        // 添加文档 创建索引
        indexWriter.addDocument(document);
        indexWriter.close();
    }

    /** 索引删除 */
    @Test
    public void  indexDelete()throws  Exception {
        // 创建分词器
        Analyzer analyzer = new StandardAnalyzer();
        // 创建Directory 流对象
        Directory directory = FSDirectory.open(Paths.get("D:/lucene/index3"));
        IndexWriterConfig config = new IndexWriterConfig(analyzer);
        // 创建索引写入对象
        final IndexWriter indexWriter = new IndexWriter(directory, config);
        indexWriter.deleteDocuments(new Term("desc", "one world one dream"));
        indexWriter.deleteAll();
        final Document document = new Document();
        document.add(new TextField("handsome", "fucking", Field.Store.YES));
        indexWriter.addDocument(document);
        indexWriter.close();
    }

    /** 索引更新 */
    @Test
    public void indexUpdate() throws Exception{
        Analyzer analyzer = new StandardAnalyzer();
        // 创建Directory 流对象
        Directory directory = FSDirectory.open(Paths.get("D:/lucene/index3"));
        IndexWriterConfig config = new IndexWriterConfig(analyzer);
        // 创建索引写入对象
        final IndexWriter indexWriter = new IndexWriter(directory, config);
        // 创建Document
        Document  document = new Document();
        document.add(new TextField("id","1001", Field.Store.YES));
        document.add(new TextField("name","好好学习", Field.Store.YES));
        document.add(new TextField("desc","游戏结束该干啥干啥去", Field.Store.YES));
        // 更新
        indexWriter.updateDocument(new Term("name","game"),document);
        indexWriter.close();
    }
}
