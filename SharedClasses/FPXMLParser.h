//
//  FPXMLParser.h
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 9/15/10.
//  For license see LICENSE.TXT
//

#import "FPGameProtocols.h"

@class FPXMLParser;

@protocol FPXMLParserDelegate <NSObject>

- (void)parser:(FPXMLParser *)parser foundObject:(id<FPGameObject>)gameObject;

@end    

@interface FPXMLParser : NSObject <NSXMLParserDelegate>
{
    id<FPXMLParserDelegate> delegate;
    id<FPGameObject> currentObject;    
    NSString *currentElement;
    int depth;
}

@property (readwrite, assign) id<FPXMLParserDelegate> delegate;

@end
