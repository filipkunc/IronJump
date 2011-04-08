//
//  FPXMLWriter.h
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 9/14/10.
//  For license see LICENSE.TXT
//

#import <Foundation/Foundation.h>


@interface FPXMLWriter : NSObject 
{
    NSMutableString *string;
    int depth;
    BOOL closedTwiceOrMore;
}

- (NSData *)data;
- (void)openElement:(NSString *)elementName;
- (void)closeElement:(NSString *)elementName;
- (void)writeFloatValue:(float)value;
- (void)writeIntValue:(int)value;
- (void)writeElementWithName:(NSString *)elementName floatValue:(float)value;
- (void)writeElementWithName:(NSString *)elementName intValue:(int)value;

@end
