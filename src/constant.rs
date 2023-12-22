macro_rules! TABLE_ROW {
    () => {
        "<tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>
                <button class=\"btn btn-danger\"
                hx-get=\"/expenses/{}/edit\"
                hx-trigger=\"edit\"
                _=\"on click
                   if .editing is not empty
                       Swal.fire({{title: 'Already Editing',
                                  showCancelButton: true,
                                  confirmButtonText: 'Yep, Edit This Row!',
                                  text:'Hey!  You are already editing a row!  Do you want to cancel that edit and continue?'}})
                       if the result's isConfirmed is false
                         halt
                       end
                       send cancel to .editing
                     end
                     trigger edit\">
                    Edit
                </button>
            </td>
        </tr>"
    };
}

macro_rules! EDITABLE_TABLE_ROW {
    () => {
        "<tr hx-trigger='cancel' class='editing' hx-get=\"/expenses/{}\">
            <td><input type='date' name='date' value='{}'></td>
            <td><input type='text' name='description' value='{}'></td>
            <td><input type='number' name='price' value='{}'></td>
            <td><select name='expense_type'>
                <option value='Food'>Food</option>
                <option value='Transport'>Transport</option>
                <option value='Health'>Health</option>
                <option value='Education'>Education</option>
                <option value='Entertainment'>Entertainment</option>
                <option value='Other'>Other</option>
            </select></td>
            <td><input type='checkbox' name='is_essencial' {}></td>
            <td>
                <button class=\"btn btn-danger\" hx-get=\"/expenses/{}\">
                  Cancel
                </button>
                <button class=\"btn btn-danger\" hx-put=\"/expenses/{}\" hx-ext=\"json-enc\" hx-include=\"closest tr\">
                  Save
                </button>
            </td>
        </tr>"
    };
}

pub(crate) use EDITABLE_TABLE_ROW;
pub(crate) use TABLE_ROW;
