package io.matthew.lucene.test;

import io.matthew.lucene.bean.Book;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.*;
import org.apache.lucene.index.DirectoryReader;
import org.apache.lucene.index.IndexWriter;
import org.apache.lucene.index.IndexWriterConfig;
import org.apache.lucene.queryparser.classic.QueryParser;
import org.apache.lucene.search.IndexSearcher;
import org.apache.lucene.search.Query;
import org.apache.lucene.search.ScoreDoc;
import org.apache.lucene.search.TopDocs;
import org.apache.lucene.search.similarities.ClassicSimilarity;
import org.apache.lucene.store.Directory;
import org.apache.lucene.store.FSDirectory;
import org.apache.lucene.store.MMapDirectory;
import org.junit.Test;

import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;

public class LuceneIndexTest {

    @Test
    public void testCreateIndex() throws Exception {
        // 1. 采集数据
        final ArrayList<Book> books = new ArrayList<>();
        final Book lucene = Book.newBuilder().setId(1).setName("Lucene").setPrice(100.45f)
                .setDesc("Lucene Core is a Java library providing powerful indexing and search features, as well as spellchecking, " +
                        "hit highlighting and advanced analysis/tokenization capabilities. The PyLucene sub project provides Python bindings for Lucene Core. ").build();
        books.add(lucene);

        Book solr = Book.newBuilder().setId(2).setName("Solr").setPrice(320.45f)
                .setDesc("Solr is highly scalable, providing fully fault tolerant distributed indexing, search and analytics. " +
                        "It exposes Lucene's features through easy to use JSON/HTTP interfaces or native clients for Java and other languages. ").build();
        books.add(solr);

        Book hadoop = Book.newBuilder().setId(3).setName("Hadoop").setPrice(620.45f)
                .setDesc("The Apache Hadoop software library is a framework that allows for the distributed processing of large data sets across clusters of computers using simple programming models.").build();
        books.add(hadoop);

        //2.创建Document 文档对象
        List<Document> documents  = new ArrayList<>();
        for (Book book : books) {
            Document  document  = new Document();
            // 给document 添加Field
            document.add(new TextField("id",book.getId().toString(), Field.Store.YES));
            document.add(new TextField("name",book.getName(),Field.Store.YES));
            document.add(new TextField("price",book.getPrice().toString(),Field.Store.YES));
            document.add(new TextField("desc",book.getDesc(),Field.Store.YES));
            documents.add(document);
        }
        //3.创建Analyzer 分词器 对文档进行分词
        final StandardAnalyzer analyzer = new StandardAnalyzer();
        // 创建Directory   和 IndexWriterConfig 对象
        final FSDirectory directory = FSDirectory.open(Paths.get("D:/lucene/index"));
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


    @Test
    public void testSearchIndex() throws Exception {
        //1.创建Query 搜索对象
        // 创建分词器
        final StandardAnalyzer analyzer = new StandardAnalyzer();
        // 创建搜索解析器
        final QueryParser queryParser = new QueryParser("id", analyzer);
        // 创建搜索对象
        final Query parse = queryParser.parse("desc: java OR name:solr");
        //2.创建Directory 流对象  指定索引库位置
        //Directory directory = FSDirectory.open(Paths.get("D:/lucene/index"));
        //Directory directory = SimpleFSDirectory.open(Paths.get("D:/lucene/index"));
        //Directory directory = NIOFSDirectory.open(Paths.get("D:/lucene/index"));
        Directory directory = MMapDirectory.open(Paths.get("D:/lucene/index"));
        // 3. 创建索引读取对象
        final DirectoryReader indexReader = DirectoryReader.open(directory);
        // 4. 创建索引搜索对象
        final IndexSearcher indexSearcher = new IndexSearcher(indexReader);
        indexSearcher.setSimilarity(new ClassicSimilarity());
        System.out.println(indexSearcher.getSimilarity(true));
        //System.out.println(indexSearcher.getSimilarity(false));

        //5. 执行搜索 返回结果集 TopDocs
        final TopDocs topDocs = indexSearcher.search(parse, 2);
        System.out.println("查询到的数据总条数:" + topDocs.totalHits);
        // 获取排序的文档
        final ScoreDoc[] docs = topDocs.scoreDocs;
        // 6.解析结果集
        for (ScoreDoc scoreDoc : docs) {
            // 获取文档id
            final int docId = scoreDoc.doc;
            final Document doc = indexSearcher.doc(docId);
            System.out.println("score:" + scoreDoc.score);

            System.out.println("docId:" + docId);
            System.out.println("bookId:" + doc.get("id"));
            System.out.println("name:" + doc.get("name"));
            System.out.println("price:" + doc.get("price"));
            System.out.println("desc:" + doc.get("desc"));
            System.out.println();
        }
        indexReader.close();
    }


    @Test
    public void createIndex() throws Exception {
        // 1. 采集数据
        final ArrayList<Book> books = new ArrayList<>();
        final Book lucene = Book.newBuilder().setId(1).setName("Lucene").setPrice(100.45f)
                .setDesc("Lucene Core is a Java library providing powerful indexing and search features, as well as spellchecking, " +
                        "hit highlighting and advanced analysis/tokenization capabilities. The PyLucene sub project provides Python bindings for Lucene Core. ").build();
        books.add(lucene);

        Book solr = Book.newBuilder().setId(11).setName("Solr").setPrice(320.45f)
                .setDesc("Solr is highly scalable, providing fully fault tolerant distributed indexing, search and analytics. " +
                        "It exposes Lucene's features through easy to use JSON/HTTP interfaces or native clients for Java and other languages. ").build();
        books.add(solr);

        Book hadoop = Book.newBuilder().setId(21).setName("Hadoop").setPrice(620.45f)
                .setDesc("The Apache Hadoop software library is a framework that allows for the distributed processing of large data sets across clusters of computers using simple programming models.").build();
        books.add(hadoop);

        // 2.将采集到的数据封装到Document对象中
        final ArrayList<Document> docList = new ArrayList<>();
        Document document;
        for (Book book : books) {
            document = new Document();
            // IntPoint 分词 索引 不存储 存储结合 StoredField
            final Field id = new IntPoint("id", book.getId());
            System.out.println(id.fieldType().tokenized() + ":" + id.fieldType().stored());
            Field id_v = new StoredField("id", book.getId());
            // 分词、索引、存储 TextField
            Field name = new TextField("name", book.getName(), Field.Store.YES);
            //  分词、索引、不存储 但是是数字类型，所以使用FloatPoint
            Field price = new FloatPoint("price", book.getPrice());
            // 分词、索引、不存储 TextField
            Field desc = new TextField("desc", book.getDesc(), Field.Store.YES);

            // 将field 域设置到Document 对象中

            document.add(id);
            document.add(id_v);
            document.add(name);
            document.add(price);
            document.add(desc);
            docList.add(document);
        }
        // 3. 创建Analyzer 分词器 对文档进行分词
        final StandardAnalyzer analyzer = new StandardAnalyzer();
        // 创建Directory   和 IndexWriterConfig 对象
        final FSDirectory directory = FSDirectory.open(Paths.get("D:/lucene/index2"));

        final IndexWriterConfig indexWriterConfig = new IndexWriterConfig(analyzer);

        // 4. 创建IndexWriter 写入对象
        IndexWriter indexWriter = new IndexWriter(directory, indexWriterConfig);

        // 添加文档对象
        for (Document doc : docList) {
            indexWriter.addDocument(doc);
        }
        // 释放资源
        indexWriter.close();
    }
}
