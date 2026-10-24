use bingeget::engine::downloader::{build_download_args, parse_progress_line, DownloadConfig, DownloadTaskSpec};

#[test]
fn test_build_download_args_with_jellyfin_naming() {
    let spec = DownloadTaskSpec {
        m3u8_url: "https://bfikuncdn.com/20260809/XpZySbA2/index.m3u8".to_string(),
        custom_title: "繁花".to_string(),
        season: 1,
        episode: 5,
        save_dir: "/media/TV/繁花".to_string(),
        referer: "https://chinaq.fun/tv-cn/202614235/ep5.html".to_string(),
    };

    let config = DownloadConfig {
        threads: 16,
        tmp_dir: "/tmp/bingeget_chunks".to_string(),
        retry_count: 5,
        binary_path: None,
    };

    let args = build_download_args(&spec, &config);

    // Verify key arguments
    assert_eq!(args[0], spec.m3u8_url);
    
    // Check save-name format: "繁花 - S01E05"
    let save_name_pos = args.iter().position(|a| a == "--save-name").expect("must have --save-name");
    assert_eq!(args[save_name_pos + 1], "繁花 - S01E05");

    // Check save-dir
    let save_dir_pos = args.iter().position(|a| a == "--save-dir").expect("must have --save-dir");
    assert_eq!(args[save_dir_pos + 1], "/media/TV/繁花");

    // Check thread count
    let threads_pos = args.iter().position(|a| a == "--thread-count").expect("must have --thread-count");
    assert_eq!(args[threads_pos + 1], "16");

    // Check mp4 muxing
    let mux_pos = args.iter().position(|a| a == "-M").expect("must have -M");
    assert_eq!(args[mux_pos + 1], "format=mp4");

    // Check cleanup, no-log, and auto-select
    assert!(args.contains(&"--auto-select".to_string()));
    assert!(args.contains(&"--del-after-done".to_string()));
    assert!(args.contains(&"--no-log".to_string()));

    // Check headers
    let has_referer = args.iter().any(|a| a.starts_with("Referer: https://chinaq.fun/"));
    assert!(has_referer, "must include Referer header");
}

#[test]
fn test_build_download_args_double_digit_season() {
    let spec = DownloadTaskSpec {
        m3u8_url: "https://example.com/test.m3u8".to_string(),
        custom_title: "老友記".to_string(),
        season: 10,
        episode: 24,
        save_dir: "/media/TV".to_string(),
        referer: "".to_string(),
    };

    let config = DownloadConfig::default();
    let args = build_download_args(&spec, &config);

    let save_name_pos = args.iter().position(|a| a == "--save-name").unwrap();
    assert_eq!(args[save_name_pos + 1], "老友記 - S10E24");
    assert!(args.contains(&"--no-log".to_string()));
}

#[test]
fn test_parse_progress_line() {
    let line1 = "Progress: 45.2%  Speed: 18.50MB/s  ETA: 00:00:32";
    let p1 = parse_progress_line(line1).expect("Should parse progress line");
    assert_eq!(p1.percent, 45.2);
    assert_eq!(p1.speed, "18.50MB/s");
    assert_eq!(p1.eta, "00:00:32");

    // Case 2: standard n-m3u8dl-re output format (e.g. 80.50% 12.3MB/s 00:01:10)
    let line2 = "[12:34:56] 80.5% 12.3MB/s 00:01:10";
    let p2 = parse_progress_line(line2).expect("Should parse n-m3u8dl-re line");
    assert_eq!(p2.percent, 80.5);
    assert_eq!(p2.speed, "12.3MB/s");
    assert_eq!(p2.eta, "00:01:10");

    // Case 3: n-m3u8dl-re initial state with negative rate and dashed ETA
    let line3 = "0.0% -0.00Bps --:--:--";
    let p3 = parse_progress_line(line3).expect("Should parse initial n-m3u8dl-re line");
    assert_eq!(p3.percent, 0.0);
    assert_eq!(p3.speed, "-0.00Bps");
    assert_eq!(p3.eta, "--:--:--");

    // Case 4: MBps unit
    let line4 = "55.4% 14.25MBps 00:00:45";
    let p4 = parse_progress_line(line4).expect("Should parse MBps line");
    assert_eq!(p4.percent, 55.4);
    assert_eq!(p4.speed, "14.25MBps");
    assert_eq!(p4.eta, "00:00:45");

    // Case 5: KBps unit with dashed ETA
    let line5 = "12.1% 850.5KBps --:--:--";
    let p5 = parse_progress_line(line5).expect("Should parse KBps line");
    assert_eq!(p5.percent, 12.1);
    assert_eq!(p5.speed, "850.5KBps");
    assert_eq!(p5.eta, "--:--:--");

    // Case 6: with ANSI escape codes
    let line6 = "\x1b[2K 99.9% 2.10MB/s 00:00:01";
    let p6 = parse_progress_line(line6).expect("Should parse line with ANSI escapes");
    assert_eq!(p6.percent, 99.9);
    assert_eq!(p6.speed, "2.10MB/s");
    assert_eq!(p6.eta, "00:00:01");

    // Irrelevant log
    let line7 = "Writing chunks to disk...";
    assert!(parse_progress_line(line7).is_none());
}

#[test]
fn test_progress_stream_with_carriage_returns() {
    let output_chunks = vec![
        "0.0% 0.00Bps --:--:--\r15.5% 2.50MBps 00:01:00\r",
        "50.0% 5.00MBps 00:00:30\r\n",
        "100.0% 1.00MBps 00:00:00\n",
    ];

    let mut line_buf = Vec::new();
    let mut parsed_progress = Vec::new();

    for chunk in output_chunks {
        for &b in chunk.as_bytes() {
            if b == b'\r' || b == b'\n' {
                if !line_buf.is_empty() {
                    let line_str = String::from_utf8_lossy(&line_buf);
                    if let Some(prog) = parse_progress_line(&line_str) {
                        parsed_progress.push(prog);
                    }
                    line_buf.clear();
                }
            } else {
                line_buf.push(b);
            }
        }
    }

    assert_eq!(parsed_progress.len(), 4);
    assert_eq!(parsed_progress[0].percent, 0.0);
    assert_eq!(parsed_progress[1].percent, 15.5);
    assert_eq!(parsed_progress[2].percent, 50.0);
    assert_eq!(parsed_progress[3].percent, 100.0);
}
