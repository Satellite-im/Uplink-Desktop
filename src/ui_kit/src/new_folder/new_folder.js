const new_folder_input_element = document.getElementById("new-folder-input")
var input_value = ""

new_folder_input_element.addEventListener("input", (_) => {
  input_value = new_folder_input_element.value
})

var show_input_and_focus = function (_) {
  new_folder_input_element.style.display = "block"
  new_folder_input_element.value = input_value
  new_folder_input_element.focus()
}

show_input_and_focus()

document.addEventListener("click", (event) => {
  if (new_folder_input_element.style.display === "block") {
    show_input_and_focus()
  }

  //   const new_folder = document.getElementById("new-folder-id")
  //   if (!new_folder.contains(event.target)) {
  //     new_folder_input_element.value = ""
  //     new_folder_input_element.style.display = "none"
  //     new_folder.style.display = "none"
  //     removeEventListener("click")
  //   }
})
