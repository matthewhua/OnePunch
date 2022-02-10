package io.matthew.lucene.test;

import org.apache.lucene.analysis.Analyzer;
import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.Document;
import org.apache.lucene.document.IntPoint;
import org.apache.lucene.index.DirectoryReader;
import org.apache.lucene.index.IndexReader;
import org.apache.lucene.index.Term;
import org.apache.lucene.queryparser.classic.MultiFieldQueryParser;
import org.apache.lucene.queryparser.classic.ParseException;
import org.apache.lucene.queryparser.classic.QueryParser;
import org.apache.lucene.queryparser.flexible.standard.StandardQueryParser;
import org.apache.lucene.search.*;
import org.apache.lucene.search.spans.SpanNearQuery;
import org.apache.lucene.search.spans.SpanQuery;
import org.apache.lucene.search.spans.SpanTermQuery;
import org.apache.lucene.store.Directory;
import org.apache.lucene.store.FSDirectory;
import org.junit.Test;
import org.wltea.analyzer.lucene.IKAnalyzer;

import java.nio.file.Paths;

public class QueryTest {


    private void doSearch(Query query) throws Exception {
        // 创建Directory 流对象
        Directory directory = FSDirectory.open(Paths.get("D:/lucene/index5"));
        // 创建IndexReader
        IndexReader indexReader = DirectoryReader.open(directory);
        IndexSearcher searcher = new IndexSearcher(indexReader);
        // 获取 TopDocs
        final TopDocs topDocs = searcher.search(query, 10);
        System.out.println("查询 索引条数:" + topDocs.totalHits);
        final ScoreDoc[] docs = topDocs.scoreDocs;
        // 解析结果集
        for (ScoreDoc scoreDoc : docs) {
            final int docId = scoreDoc.doc;
            final Document document = searcher.doc(docId);

            System.out.println("docId" + docId);
            System.out.println("bookId" + document.get("id"));
            System.out.println("name:"+document.get("name"));
            System.out.println("price:"+document.get("price"));
            System.out.println("desc:"+document.get("desc"));
        }
        indexReader.close();
    }

    @Test
    public void testSearchTermQuery() throws Exception {
        // 创建TermQuery 搜索对象
        final TermQuery query = new TermQuery(new Term("name", "matthew"));

        doSearch(query);
    }

    @Test
    public void  testSearchBooleanQuery() throws Exception {
        // 创建两个 TermQuery搜索对象
        final TermQuery query1 = new TermQuery(new Term("name", "matthew"));
        final TermQuery query2 = new TermQuery(new Term("desc", "帅哥")); //只能搜索里面分好的词
        // 创建BooleanQuery 搜索对象, 组合查询操作
        final BooleanQuery.Builder boolQuery = new BooleanQuery.Builder();
        // 组合条件
        // 第一个参数, 查询条件, 第二个参数, 组合方式
        boolQuery.add(query1, BooleanClause.Occur.MUST);
        boolQuery.add(query2, BooleanClause.Occur.MUST);

        doSearch(boolQuery.build());
    }

    /** 短语查询 */
    @Test
    public void testSearchPhraseQuery() throws Exception {
        final PhraseQuery query = new PhraseQuery( 3,"desc", "我是", "帅哥"); // 要按顺序来
        doSearch(query);
    }

    //  跨度查询
    @Test
    public void testSearchSpanNearQuery() throws Exception {
        final SpanTermQuery tq1 = new SpanTermQuery(new Term("desc", "我是"));
        final SpanTermQuery tq2 = new SpanTermQuery(new Term("desc", "帅哥"));
        final SpanNearQuery spanNearQuery = new SpanNearQuery(new SpanQuery[]{tq1, tq2}, 3, true);  //我是绝壁 大帅哥 中间差3位
        doSearch(spanNearQuery);
    }

    /** 模糊查询 */
    @Test
    public void testSearchLikeQuery() throws Exception {
        // WildcardQuery：通配符查询  *表示0个或多个字符，?表示1个字符，\是转义符。
       /* WildcardQuery wildcardQuery = new WildcardQuery( new Term("name", "mat*"));
        doSearch(wildcardQuery);*/
        final FuzzyQuery fuzzyQuery = new FuzzyQuery(new Term("name", "mattehew"), 2); //输错也可以搜索
        doSearch(fuzzyQuery);
    }

    @Test
    public void testSearchNumQuery() throws Exception {
        final Query query = IntPoint.newRangeQuery("id", 1, 10);
        doSearch(query);
    }

    @Test
    public void testQueryParser() throws Exception {
        // Analyzer analyzer  = new StandardAnalyzer(); 检索不出来
        final IKAnalyzer analyzer = new IKAnalyzer();
        final QueryParser queryParser = new QueryParser("desc", analyzer);
        // 构建搜索对象
        final Query query = queryParser.parse("desc:帅哥 AND name:matthew");
        doSearch(query);
    }

    @Test
    public void testSearchMultiQueryParser() throws Exception {
        // 创建分词器
        Analyzer analyzer = new IKAnalyzer();
        // 1. 创建MultiFieldQueryParser搜索对象
        String[] fields = {"name", "desc"};
        final MultiFieldQueryParser multiFieldQueryParser = new MultiFieldQueryParser(fields, analyzer);
        // 创建搜索对象
        final Query query = multiFieldQueryParser.parse("帅哥");
        // 打印生成的搜索语句
        System.out.println(query);
        // 执行搜索
        doSearch(query);
    }

    @Test
    public void testSearchQueryParser() throws Exception {
        final StandardAnalyzer analyzer = new StandardAnalyzer();
        final StandardQueryParser standardQueryParser = new StandardQueryParser(analyzer);
        final Query query = standardQueryParser.parse("desc:我 AND name:matthew", "desc");

        //通配符匹配 建议通配符在后 通配符在前效率低
        // query = parser.parse("name:L*","desc");
        // query = parser.parse("name:L???","desc");

        // 模糊匹配 query = parser.parse("lucene~","desc");
        // 区间查询 query = parser.parse("id:[1 TO 100]","desc");

        // 跨度查询 ~2表示词语之间包含两个词语
        // query= parser.parse("\"lucene java\"~2","desc");
        doSearch(query);
    }



}

