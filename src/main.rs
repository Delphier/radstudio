use radstudio;

fn main() {
    let installs = radstudio::find().unwrap();
    for (index, install) in installs.iter().enumerate() {
        println!("{}. {}", index + 1, install.product_info().full_name());
        println!("{}", install.product_info());
    }
}
