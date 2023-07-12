

pub fn print_process_tree(items:  &[Vec<String>], id: String,  depth: usize) {
    let padding = "  ".repeat(depth);
    let result = items.iter().find(|&v| v[0] == id);
    let mut p_name = "";
    if let Some(vec) = result {
        if let Some(name) = vec.get(11) {
            p_name= name;
        }
    }
    if id=="1" {

        println!("{}──({})",  p_name, id);
    } else{

        println!("{}└──── {} ({})", padding, p_name, id); 
    }
   
    let children: Vec<String> = items
        .iter()
        .filter(|&v| v[2] == id)  // Filter out sub-vectors with third element != "3"
        .map(|v| v[0].clone())  // Map each remaining sub-vector to its first element
        .collect();  // Collect the results into a new vector
    for child in children {
        print_process_tree(items, child, depth + 4);
    }

}
