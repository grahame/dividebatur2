dividebatur2: process single-transferable-vote elections

Currently supports the following STV election types:

 - Australian Senate under the Commonwealth Electoral Act (1918) (post 2015 voting reforms)

dividebatur2 is a work-in-progress, porting [dividebatur](https://github.com/grahame/dividebatur) to the Rust 
programming language. If you're after something more mature, check this out. The primary motivation for the
rewrite is improvements in performance, and in correctness and maintainability. dividebatur2 is currently
clocking in ~ 15x faster than the original Python implementation.

For a high level overview of what this does, check this blog post:

http://blog.angrygoats.net/2014/01/25/counting-the-west-australian-senate-election/

For the legislation defining the system we are implementing, see the `legislation` directory of the repository.
The counting algorithm is defined in Section 273. Other sections define the rules around formality of ballots.

## License

Copyright 2013-18, The Dividebatur Authors

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

See the file AUTHORS for a list of contributors to this software project.

The data located under `aec_data/` is licensed under a 
Creative Commons Attribution 3.0 Australia Licence (CC BY 3.0), and is 
© Commonwealth of Australia. Full License terms are on the
[AEC website](http://aec.gov.au/footer/Copyright.htm)

## Contributing

Contributions are welcomed, please feel free to send in pull requests.

Please abide by [The Rust Code of Conduct](https://www.rust-lang.org/en-US/conduct.html)
when contributing to or otherwise engaging this project.

## Usage

For now, check out `dividebatur` in the parent directory to this repository. Follow the
instructions in that repository to make sure you've got all the vote data. Then you're
good to go with a simple:

```
$ cargo run --release
```
